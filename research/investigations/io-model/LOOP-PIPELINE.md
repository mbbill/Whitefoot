<!-- Header and §9 added when the design was landed in the repository; §§0-8
     below are the scratch design record, verbatim. -->

> **This is the design record for batches 0089 and after.** It was written
> against `integration/2026-08-27` at `79b29665` and landed in batch 0089
> (`docs/done/0089-loop-pipeline-batch0.md`), which implements its §7 "Batch 0"
> and nothing else.
>
> **§§0-8 are the design as written, before any probe ran. §9 is what the five
> probes of §5.5 measured, appended.** The numbers in §0.1 and §5 are therefore
> predictions and the numbers in §9 are measurements; where they disagree, §9
> is the fact and §0.1 is superseded. They are kept side by side deliberately —
> which predictions held is itself evidence — but a reader taking a number out
> of this document should take it from §9. Every Linux number in §9 comes from
> a qemu-virtualised container and is provisional until the real-Linux CI run.

# The staged loop pipeline, with privatization by proof

Final design, 2026-08-27. Synthesis of designs A (`privatize-by-proof`),
B (`allocation-ring`), and C (`explicit-window-form`) and of the two judge
reports. Written against a worktree of branch
`integration/2026-08-27` at `79b29665`. Every compiler and spec citation below
was re-read in that tree by the synthesizer, not taken from the source designs.

Written for a senior engineer who will implement it without asking questions.
Where something is unproven, it says so. Where a shape must be refused, it
shows the program.

---

## 0. Verdict, and what changed from the three inputs

**The winner is B's chassis with A's proof grafted on top, delivered in that
order.** B supplies the schedule, the permission shape, the storage discipline,
and — decisively — the observation that no stackless generalization is needed.
A supplies the one thing B declines: a derived byte-range analysis that makes
the *byte-unchanged* `many_files_narrow.wf` fast, which is the owner's stated
falsifier. C supplies the presentation that makes the missing case visible (the
four-disposition table), the sharpest must-refuse program, and the refusal
table for writer-visible forms. C's own deliverable — a mandatory extent column
on every buffer-typed effect path — is dropped entirely; it rejects the
benchmark's own `name_at` (§2.7) and costs more than the derived analysis that
needs no source change.

Both judges' rankings collapse to the same engineering answer once the
sequencing is fixed: **build B, then add A.** Judge 2 states it in one line
("Build B's pipeline first, then add this to privatize a hoisted buffer and
reach the unchanged source"); judge 1's reason for ranking A first — that only
A targets the byte-unchanged program — is satisfied by that sequence, and every
one of judge 1's four fatal flaws against A is either fixed here or recorded as
an open question.

### 0.1 The corrected measurement record

This is the most important correction in the document, and all three designs
got some part of it wrong. Verified in this tree:

- `research/experiments/io-completion-bench/uring_baseline.h:189` issues **one**
  `io_uring_enter` per batch of `pending` SQEs. The native ring baseline is
  already batched, and it does its `openat` and `close` as ordinary blocking
  syscalls (`uring_baseline.h:169,178,214`). So `N.uring32`'s 53.20 ms of
  system time is **not** doorbell overhead.
- `N.direct` performs 8,192 x (openat + pread + close) = 24,576 blocking
  syscalls for **20.08 ms** of system time — 0.82 us each
  (`RESULTS.md:285`).
- `N.uring32` performs the same opens and closes blocking (~13 ms by that rate)
  plus 8,192 batched ring reads, for **53.20 ms** — imputing ~4.9 us of CPU per
  ring read (`RESULTS.md:293`).
- `C.wide8` performs 24,576 ring operations with **one `io_uring_enter` per
  operation** (`wf_linux_kick_locked` is called inside the submission lock on
  every submit, `linux_io_uring.c:653`) for **61.42 ms** — 2.5 us each
  (`RESULTS.md:302`).

Three consequences, all load-bearing:

1. **B's "sys 61 -> ~25 from batched submission" is refuted**, and so is C's
   "one enter per K submissions is where a large part of that 53.2 ms goes".
   Batching removes the syscall *entry* cost, not the per-SQE processing cost.
   Honest estimate for deferring the doorbell: 24,576 x 0.3-0.8 us = **7-20 ms**,
   not 35-45. Probe A (§5.5) measures it before anything is built.
2. **On this Linux container io_uring costs about 3x a blocking syscall in
   CPU** (2.5 us/op through the ring against 0.82 us/op direct). That, and not
   depth, is why `N.pool2` (blocking syscalls on two threads, 23.31 ms sys)
   beats every ring line. Any plan to reach 40.21 ms that keeps 24,576
   operations in the ring is arithmetic that does not close. This reopens a
   runtime policy question, not a language question — §5.3 and Probe C.
3. **`C.wide8` is core-saturated**: 65.65 user + 61.42 sys = 127.07 ms of CPU
   against 119.47 ms of wall = 1.06 of 2 cores. Removing the round barrier
   alone cannot take the program below ~120 ms. Depth is necessary and nowhere
   near sufficient. A's Claim 1 ("80-90 ms from depth alone") and Claim 2
   ("40-50 ms once the fold spreads over 2 cores", which silently drops the sys
   column) are both refuted by their own table. This design does not repeat
   them.

One further observation none of the three made: `N.uring32` accounts
49.47 + 53.20 = 102.67 ms of CPU inside 82.46 ms of wall on a nominally
single-threaded program. That is 1.24 cores, and the only place it can come
from is kernel-side `io-wq` workers doing ring work off the submitting CPU. So
a deeper ring may buy kernel-side parallelism that `C.wide8` at depth 8 with a
barrier does not get. This is **speculation from an accounting artifact, not a
measurement**, and it is listed as an upside that Probe A will confirm or kill.

---

## 1. The mechanism

### 1.1 In one paragraph

A loop body that performs I/O is cut at its first may-suspend submission into
an **ordered prologue** P (everything up to and including that submission: the
per-iteration allocations, the name rendering, the `reserve_file` that takes
and returns a short unique factory loan, the submission itself, and every edge
that leaves the loop) and a **remainder** E (the joins, the outcome matches,
the subsequent submissions, the fold, and the accumulator writes). The compiler
gives every place the body touches that is rooted outside the loop exactly one
of four dispositions — read-only, serialized, privatized, or denied — and
grants the staged schedule only when no place lands in `denied`. The runtime
then keeps K iterations in flight: the owner lane runs prologues in index order
and never joins at the loop back-edge; each slot advances one stage per visit
through a driver loop; the pure per-iteration fold is handed to a compute lane
through the existing `wf__par_claim` path; accumulator writes commit in index
order. Every `buffer_new` in the body whose value the body releases unread
becomes one of K ring slots allocated once at loop entry, and its
compiler-derived release becomes a contract-bounded *restore* instead of a
`free`, so per-iteration allocation costs nothing. A second stage of the work
adds a derived byte-range analysis that lets a *hoisted* scratch buffer be
privatized when — and only when — every byte an iteration reads out of it was
written earlier in that same iteration; that stage is what reaches
`many_files_narrow.wf` with its bytes unchanged. K comes from the runtime, once
per loop entry, exactly as `wf__par_split_budget` is asked once today. No
writer-visible depth, window, batch, task, future, callback, or attribute is
added anywhere.

### 1.2 In code, stage 1: the per-iteration-allocation form

This is what an AI writes when it is not hand-optimizing. Commit it as
`research/experiments/io-completion-bench/programs/many_files_loop.wf`.
`name_at`, `render_u64` and `fold_bytes` are byte-identical to the existing
bench sources.

```whitefoot
command fn main(
  command.cwd as cwd: own DirectoryRead,
  command.stdout as out: own Output,
  command.files as files: own FileFactory
) -> status: own ExitStatus
reads(cwd, out, files), writes(cwd, out, files), allocates(heap) {
  doc "Opens and reads the generated file set and publishes one folded checksum.";
  let line = buffer_new(64_u64, 0_u8);
  let sum = 0_u64;
  let bytes = 0_u64;
  for @scan index in 0_u64..8192_u64 {
    let name = buffer_new(16_u64, 0_u8);          // iteration-own
    let data = buffer_new(65536_u64, 0_u8);       // iteration-own
    region 'name {
      let rendered = name_at<'name>(name: &uniq 'name name, index: index);
    }
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd,
                                name: &'n name, start: 0_u64, end: 10_u64) {
          Ok(value: handle) => {
            region 'h { region 'd {
              match read_at<'h, 'd>(file: &'h handle, destination: &uniq 'd data,
                                    file_offset: 0_u64, start: 0_u64, end: 65536_u64) {
                ReadBytes(next: produced) => {
                  let digest = 0_u64;
                  region 'fold {
                    set digest = fold_bytes<'fold>(source: &'fold data,
                                                   produced: produced, seed: 0_u64);
                  }
                  let weight = index +wrap 1_u64;
                  set digest = digest *wrap weight;
                  set sum = sum +wrap digest;
                  set bytes = bytes +wrap produced;
                }
                ReadEnd() => { }
                ReadFailed(error: problem) => { }
              }
            } }
          }
          Err(error: problem) => { }
        }
      }
    }
  }
  /* render and publish, unchanged */
}
```

### 1.3 In code, stage 2: the shipped program, byte-unchanged

`research/experiments/io-completion-bench/programs/many_files_narrow.wf` lines
95-142, unmodified. Its differences from §1.2 are the ones that matter:

- `loop @scan` with a hand-carried `index` and a `break`, not a counted `for`.
- `name` and `data` are `buffer_new`d **above** the loop, so they are enclosing
  places, not iteration-own.
- Two accumulators, `sum` and `bytes`.
- Both system calls are `match` scrutinees, not `let` bindings.

Each of those four is handled, and each needs saying because C concluded that
the first is a wall (it is not: the staged permission's unit is the iteration,
not an index subrange, so no trip count and no induction variable is recovered
— §2.9) and B concluded that the second is a wall (it is not, once §2.6's
derived byte ranges exist).

The compiler's per-place classification of `loop @scan`:

| place | disposition | why | what lowering does |
|---|---|---|---|
| `cwd` | **read-only** | nothing writes it; `open_file`'s retained `&cwd` is shared, and two overlapping shared loans deny nothing ([PAR-1]) | nothing |
| `files` | **serialized** | every access is `reserve_file`'s, a `never-suspends` call in P; the loan ends when that inline operation returns ([SYS-10], spec:2570) | nothing |
| `name` | **serialized** | `name_at`'s exclusive loan dies in P; `open_file`'s `&name` is released *before target transfer* once §4.3's milestone and §3.6's per-slot path copy land | nothing, once the record owns its path bytes |
| `data` | **privatized** | iteration-private by proof (§2.6): `read_at` defines `[0, produced)` and `fold_bytes` reads exactly `[0, produced)` | K private copies of the same length |
| `sum`, `bytes` | **accumulator-free ordered writes** | written in E, committed in index order on the owner lane | one per-slot digest, committed in order |
| `index` | **carried datum** | `index +wrap 1_u64` needs nothing a suspension produces; the recurrence closes in P | hoisted above the submission; E reads the slot's saved copy |

Note what is *not* here: no associativity condition, no identity element, no
combination tree. `set sum = sum +wrap digest` stays an ordinary source-order
write, because retirement is index-ordered. This is B's contribution and it is
strictly more general than [PAR-2]'s accumulator apparatus — it admits
non-associative folds, float folds, and `Result` routes that [PAR-2]'s admitted
operation set can never reach. `LoopDenial::ManyAccumulators`
(`compiler/src/semantic/loop_permission.rs:229`) stays exactly as it is, and
A's proposed [PAR-2] multi-accumulator amendment is **not needed**.

### 1.4 The K-in-flight timeline

K = 4 for legibility. `Pi` = prologue of iteration i (guard, allocations,
`name_at`, `reserve_file`, submit open). `~~~` = target-owned. `Jo`/`Jr`/`Jc` =
join of that slot's open / read / close token. `ri` = submit read i.
`Fi` = the handed-out fold. `Ci` = index-ordered commit of `sum` and `bytes`.

```text
                    t ->
owner lane  P0 P1 P2 P3 | Jo0 r0 | Jo1 r1 | Jo2 r2 | Jo3 r3 | Jr0 F0^ P4 | Jr1 F1^ P5 | C0 Jc0 | C1 …
             |  |  |  |     |        |        |        |        |    |       |
slot 0    ───┴~~~~~~~~~~~~~~┘        |        |        |     ~~~┘    |       |     [restore, reuse as 4]
slot 1    ──────┴~~~~~~~~~~~~~~~~~~~~┘        |        |        ~~~~~┘       |     [restore, reuse as 5]
slot 2    ─────────┴~~~~~~~~~~~~~~~~~~~~~~~~~~┘        |
slot 3    ────────────┴~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~┘
compute lane                                                    F0        F1        …

outstanding:  1  2  3  4      4        4        4        4        4         4
```

Steady state: **K target operations continuously outstanding, one join per
stage advance, no round barrier anywhere.** Contrast today's `wide8`:

```text
owner  o0..o7  J(all 8)  r0..r7  J(all 8)  F0..F7  o8..o15  J(all 8) …
```

— eight, hard join, eight, hard join, and the eight folds on the owner lane.
That barrier is the 26 % separating `C.wide8` from `N.uring8` and the reason
`N.uring32` pulls away (`RESULTS.md`, "Where the remaining distance is").

The owner lane's driver loop, precisely:

```text
loop {
  advance any slot whose current token wf__completion_file_take() reports ready
  else if a slot is free and iterations remain: run P(next) into it
  else: wf__completion_file_join() the oldest busy slot and advance it
  commit finished slots' accumulator contributions in index order
}
```

Harvesting is out of order (that is what keeps depth up); **writer-visible
retirement is index-ordered** (that is what makes the accumulators need no
algebra). Both, together, are the schedule.

---

## 2. Permission

### 2.1 The judgment, precisely

Write **L** for the loop (a `for_stmt` or a `loop_stmt`), **B** for its body,
**S** for the first `may-suspend` call in B's program order, **P** for the
prefix of B up to and including S's own argument evaluation and submission (the
*prologue*), and **E** for the rest (the *remainder*). Footprints — written,
read, operand-read — and loans are formed exactly as [PAR-1] forms them, which
is what `Program::footprint` / `call_projection`
(`compiler/src/semantic/permission.rs:1116,1244`) and `Footprint::loans`
already compute, including the loan column batch 0081 added
(`Loan { strength, place, argument }`, `permission.rs:214`).

Permission for the staged schedule holds exactly when all of the following
hold.

**(P1) The cut exists.** There is one program point `c` of B such that every
statement of B either executes before `c` on every path through B, or is
reached only through `c`. `c` is S's submission. Implementation: compute `c` on
the built IR block graph as the single-entry single-exit dominator/post-dominator
pair; **refuse when the relation is not that clean shape.** A statement-index
heuristic is not acceptable here — the natural body nests four
`region`/`match` levels deep and getting (P1) wrong in the permissive direction
breaks (P2).

**(P2) Every edge that leaves B leaves from P.** No `return_stmt`, no
`give_stmt` delivering outside B, no `break_stmt` naming L or a loop enclosing
L, and no `let_stmt` selecting `propagate_let_rhs` occurs in E.
`Survey::leaves` (`loop_permission.rs:753`) already classifies exactly these
four edges.

**(P3) Retained borrows are safe.** For every `may-suspend` call of B, every
borrow it retains past its own submission is on a place rooted in a binding B
introduces, on a privatized place, or on a place no footprint of B writes.

**(P4) Exclusive loans in E are safe.** Every exclusive loan a call of E holds
is on a place rooted in a binding B introduces or on a privatized place.

**(P5) Every place rooted outside L that B touches carries a disposition**, and
there are exactly four (§2.2). A place with no disposition denies.

**(P6) Replicated and restored storage has copy elements.** A place this
judgment privatizes, and a construction whose storage the implementation
reuses across iterations, has a copy element type.

**(P7) Fail closed.** Every call and compiler-derived release in B has a
complete target summary; an unresolved footprint element, an unresolved loan,
an unresolved extent, or a body statement form the judgment does not classify
denies permission rather than granting it. This is `Survey`'s existing
one-sided reading, quoted from its own module doc: "a missed statement would
contribute an empty footprint and *widen* permission, which is the one
direction it must never fail in."

**Why exactly these.** The schedule an implementation may take under (P1)-(P7)
is: `P(0), P(1), …` in index order, never two at once; E(i)'s stages executed
after P(i+1..i+K) may already have run; E(i)'s writes to places rooted outside
L committed in the order of i. So the only pairs that ever coexist are
(a) E(i) against P(j) for j > i, and (b) E(i) against E(j) for j != i in their
target-owned halves. (P3) and (P5) make (a) non-interfering; (P4) and (P5)
make (b) non-interfering; (P2) means no iteration ever leaves the loop from a
segment that could coexist with a later iteration's work.

### 2.2 The four dispositions

This table is C's, and it is the right way to present the judgment because it
makes a missing case visible. A's three-kinds list hid one, and that is exactly
why A's own normative text refused A's own target program (judge 1's third
fatal flaw): `name` and `files` are written by calls rooted in bindings
declared above the loop, are not accumulators, are not carried data, and are
not iteration-private, so A's condition (a) denied them. The serialized
disposition is the missing clause.

| disposition | condition | example |
|---|---|---|
| **read-only** | no footprint of B writes it, and every loan on it is shared | `cwd` |
| **serialized** | every footprint element and every loan touching it belongs to a call in P, and no loan on it is retained past S's submission | `files`, `name` |
| **privatized** | §2.6's coverage condition holds, and its element type is copy | `data` |
| **denied** | everything else | §2.3, §2.4, §2.5 |

The serialized disposition needs neither privatization nor a loan exemption,
because **prologues never overlap one another**. That is a schedule restriction
the rule text must state explicitly — it is what admits a unique factory loan,
and neither A nor B stated it (judge 2's second and sixth fatal flaws). C
stated it; the sentence is carried into §4.1 verbatim.

Two details that make `files` and `name` land here rather than in `denied`:

- **`files`.** [SYS-10] (`spec/kernel-spec.md:2570`) already fixes it: "`reserve_file`
  takes a call-scoped `&uniq FileFactory` … The factory loan ends when that
  inline operation returns, so a caller may reserve several permits through
  short sequential loans and then move those permits into independent
  long-running opens." Index-ordered prologues make those windows disjoint and
  in source order. At every program point exactly one unique loan of `files` is
  live: this is not a relaxation of [OWN-5]. (Note for the record: A attributes
  this sentence to [SYS-8]/[SYS-2]; it is [SYS-10].)
- **`name`.** `open_file` retains `&'n name` to `loan-released(name)`, which is
  published at `terminal` today. §4.3 makes that milestone `begin_submit` for
  the `name` borrow of `open_file` and `open_directory`, which is what the
  emitter already does in substance — `compiler/src/backend/emitter/system.rs:2059`
  memcpys the admitted `[start, end)` range into `%component` and NUL-terminates
  it before the host call. **That is only sound after §3.6's two latent bugs are
  fixed.** Until then, `name` is `denied` and stage 1's iteration-own `name` is
  the only admitted form.

### 2.3 A program that MUST be refused — the loop-carried scratch buffer

This is the discriminating case for privatization, and it is a *silent wrong
answer*, not a crash, so it must be a maintained program with a checked
published checksum on both the permitted and the denied side. Per the
green-is-not-coverage rule, a privatization test that passes because
privatization never fired proves nothing.

`many_files_narrow.wf` with **one token changed** — the `produced` argument to
`fold_bytes` becomes the buffer's full length:

```whitefoot
                match read_at<'h, 'd>(file: &'h handle, destination: &uniq 'd data,
                                      file_offset: 0_u64, start: 0_u64, end: 65536_u64) {
                  ReadBytes(next: produced) => {
                    let digest = 0_u64;
                    region 'fold {
                      set digest = fold_bytes<'fold>(source: &'fold data,
                                                     produced: 65536_u64,  // <- was: produced
                                                     seed: 0_u64);
                    }
```

The judgment runs identically until containment:

```text
  must-write set after read_at : { [0, produced) }   with produced <= 65536 by [ENT-3.S10]
  may-read set at fold_bytes   : { [0, 65536) }      (the callee summary [Z, produced)
                                                      with the constant substituted)
  containment goal             : 65536 <= produced   NOT derivable
```

`data` is not iteration-private; the loop is denied with a diagnostic naming
the `fold_bytes` argument node and the two intervals.

**The refusal is not conservatism.** That program is genuinely order-dependent:
a 3 KiB file leaves 61 KiB of the previous iteration in the buffer and the
published checksum depends on it. It differs from the accepted program by one
argument, the checker's answer differs, and it differs in the direction the
semantics demand. **A whole-place "write before read" privatization rule would
accept both and silently miscompile the second. Any design that claims
whole-place privatization is sound is wrong; say so out loud in review.**

### 2.4 A program that MUST be refused — the hidden loop-carried byte

The same refusal from the other side, and it is the reason the *rule* must
refuse rather than the *analysis* merely failing to prove:

```whitefoot
fn stamp['b](slot: &uniq 'b buffer<u8>, index: own u64) -> result: own unit
reads(slot), writes(slot) {
  let odd = ieq(index % 2_u64, 1_u64);
  if odd { set deref(slot)[0_u64] = 7_u8; }
  return unit;
}

let scratch = buffer_new(1_u64, 0_u8);
let sum = 0_u64;
for @scan i in 0_u64..4_u64 {
  region 's { let stamped = stamp<'s>(slot: &uniq 's scratch, index: i); }
  region 'f { set sum = sum +wrap fold_bytes<'f>(source: &'f scratch,
                                                 produced: 1_u64, seed: 0_u64); }
}
```

Sequentially `scratch[0]` takes 0, 7, 7, 7 and `sum` is 21. Privatized into
fresh per-iteration copies it takes 0, 7, 0, 7 and `sum` is 14. `stamp`'s
must-write set is `Empty` (the write is guarded and the analysis has no
must-fact for `odd`), the read's may-read set is `[0,1)`, containment fails,
`scratch` is denied. This is the case a must-write/may-read pair is designed to
catch and a whole-place rule cannot.

### 2.5 A program that MUST be refused — the fold before the read

```whitefoot
  region 'fold { set digest = fold_bytes<'fold>(source: &'fold data,
                                                produced: carried, seed: digest); }
  match read_at<'h, 'd>(file: &'h handle, destination: &uniq 'd data, ...) { … }
```

The read of `[0, carried)` is preceded on its path by no write of `data` at
all: the must-write set is empty. Denied. That is a true loop-carried
dependence and no amount of machinery should admit it.

Also permanently denied, for a different and structural reason: a body holding
a retained exclusive loan on an opaque nominal that cannot be replicated —
`directory_next(source: &uniq DirectorySource, …)`. A cursor has one position.
It is not read-only, not prologue-only, and not copyable. The denial is the
ownership model working, not a gap.

### 2.6 Iteration-privacy, and where each fact comes from

This is stage 2, and it is the only part of the design that consumes the
entailment fact state.

> **P is iteration-private in B** exactly when no statement L's continuation
> reaches reads P, and on every path through B every byte of P a footprint of B
> reads was written by an earlier footprint of B on that path.

**The summary is an interval set, not a prefix.** This corrects A's exposition,
which tracked a "covered prefix" and would silently accept a genuinely wrong
program. Concretely, for a body doing `read_at(start 0, end 32768)` then
`read_at(file_offset 32768, start 32768, end 65536)` then
`fold_bytes(produced: p2)`, the true covered set is `[0,p1) union [32768,p2)`
and the gap `[p1, 32768)` holds the previous iteration's bytes unless
`p1 >= 32768`, which [ENT-3.S10] cannot derive. A prefix-max implementation
accepts and publishes stale bytes. So:

- A **must-write** summary is an ordered set of at most four disjoint half-open
  intervals over entry-stable terms. Union of two intervals collapses to their
  convex hull only when adjacency (`w1.end >= w2.start`) is derivable at that
  point; otherwise both are kept. Overflow past four drops the excess
  intervals — the fail-closed direction for a *must* set is *smaller*.
- A **may-read** summary is likewise at most four intervals; overflow past four
  widens to `All` — the fail-closed direction for a *may* set is *larger*.
- Containment asks, for each read interval, that it lie inside the union of the
  must-write intervals, which needs adjacency facts wherever it spans two.
  Underivable containment **denies**.
- Anything unresolved is `All` on the read side and `Empty` on the write side.
  A recursive or mutually recursive callee is `All`/`Empty`.

Byte ranges come from exactly three places:

1. **A system operation**: its contract fixes them. [SYS-8]
   (`spec/kernel-spec.md:2532`) already says "On `ReadBytes(next)` exactly
   `[start, next)` may have changed and every other byte of the buffer is
   unchanged" and "On `ReadEnd` and on `ReadFailed` no byte of the buffer
   changes". §4.2 adds the *observed* side and the word **defines**, which is
   what privatization needs and "may have changed" does not supply.
2. **A direct element access** `deref(P)[e]`: the exact position `[e, e+1)`,
   widened to `[lo, hi)` by asking the closed L0 state at that node for the
   greatest lower bound on `e` and least upper bound on `e+1` expressible over
   entry-stable terms. **This is the second question asked at a node that
   already asks the first one** — the [ENT-6] SubscriptBounds obligation — which
   is the whole reason the analysis is cheap.
3. **A user call**: the callee's summary, substituted through the actual
   arguments by exactly the [EFF-2] call-boundary projection `footprint`
   already performs.

**Observing a place's length reads no byte of it.** `check_flat_length`
(`compiler/src/semantic/check/expressions/flat_storage.rs:512`) contributes
`add_read(path)` for `len(deref(b))`, so `fold_bytes` must declare
`reads(source)` even though `len` observes no byte. A private copy of the same
length answers a length read identically, so a length read can never make
privatization unsound. This sentence is what makes the natural program work.

**Discharging `data` in `many_files_narrow.wf`.** At the
`ReadBytes(next: produced)` arm the checker holds, from [ENT-3.S10]
(`spec/kernel-spec.md:2953-2955`), `0 <= produced` and `produced <= 65536`, and
from the operation contract the defined set `{ [0, produced) }`. `fold_bytes`'s
own summary, derived once from its body, is
`may_read(source) = [Z, produced)`, `must_write(source) = Empty`: at
`let byte = deref(source)[at]` the closed state derives `at < produced` from
S1's exact L0 negation of `ige(at, produced)` on the false edge established
immediately above, with no intervening [ENT-5] kill of a fact supported by
`at`; `produced` is a parameter and never a `set` target in `fold_bytes`, so it
is entry-stable. Substituting the actual `produced` gives read set
`{ [0, produced) }`, contained in `{ [0, produced) }`. The other two arms touch
`data` not at all. **`data` is iteration-private.**

**One correction to B's own framing.** B's Fact 2 states that when the scratch
places are declared outside the loop "iteration i+1 cannot even start while
iteration i is in flight, and no proof will change that — it is [OWN-5]
exclusivity, correctly applied." The first half is true of the *loan*; the
second half is overstated, and B refutes it itself two sections later by naming
the proof as its own W1 residual's closing condition. The enabling condition is
per-iteration storage **or** a proof that the place is iteration-private. This
document supplies the proof, and that is exactly what stage 2 is.

**Do not privatize `name`.** `name_at` opens with
`let sized = ige(room, 10_u64); if sized { } else { return unit; }` — a normal
return edge along which nothing is written — and its digit writes are guarded
by `ilt(position, room)` with loop-carried `position`. Its must-write set is
`Empty`, so `name` is not privatizable and never will be by this analysis.
It does not need to be: §2.2's serialized disposition plus §4.3's release
milestone admit it. (This is exactly the case C's mandatory extent column
cannot state at all — see §2.7 — and it is the strongest single argument for
deriving extents instead of declaring them.)

### 2.7 Why C's extent column is dropped

C concluded that the one writer-visible form that earns its place is a required
extent on every buffer-typed effect path (`reads(source[0..produced])`), with
`writes` extents given a must-write meaning: "[EFF-2] rejects a body with a
normal return edge along which some element of it is unwritten."

`name_at` in the benchmark has exactly such an edge. Under C's Part 2,
`writes(name[0..10])` is unstatable and `writes(name[0..0])` is refused by the
other direction, so **`name_at` does not compile** — the one form C concludes
earns its place makes the program it exists to accelerate fail to build. C does
not notice this. The derived analysis of §2.6 has no such problem: it computes
`Empty` and routes `name` through the serialized disposition instead.

That, plus a grammar production, plus a rewrite of every buffer-typed effect
row in the corpus and in the conformance evidence, plus 1,500-2,500 lines over
several versions, against zero source change for the derived route. Dropped.

### 2.8 Early typed exits

(P2) is the whole of it: an exit edge is admitted exactly when it lies in P.

- **`many_files_narrow.wf`'s `if done { break @scan; }`** has exactly
  `let done = ige(index, 8192_u64);` before it, and sits in P. **Admitted.**
  This is why the staged permission does not need L to be a `for_stmt`.
- **The narrow program's `ReadEnd()`, `ReadFailed(error: problem)` and
  `Err(error: problem)` arms** are empty and fall through. They are not exit
  edges; they are ordinary control. **Admitted**, and both opens and reads
  pipeline K deep.
- **`many_files_wide8.wf`'s `Err(error: failed_a) => { return exit_status(code: 4_u8); }`**
  sits in E. **Denied.** Correctly: with K iterations in flight, iteration i's
  decision to return is taken after i+1..i+K-1 have already submitted opens
  that sequential execution never performs, and an `openat` is an externally
  observable state transition that [PAR-1] says is not rolled back. `wide8`
  keeps exactly the [PAR-1] window overlap it has today and must be verified
  byte-unchanged (falsifier F4, §5.4).
- **A body whose first I/O error returns** gets no pipeline. On Linux, where an
  `openat` costs 0.85 us and the whole open-plus-close budget is ~11 ms of a
  119 ms program, moving the cut to the `read_at` site loses almost nothing; on
  macOS, where one `openat` costs 116 us, it loses nearly everything. **This is
  a real, named W1 residual, not a solved case.** Its closing condition is
  FIRST-PRINCIPLES §8's closed-state proof extended to a target call whose
  *result* is a fresh resource; per-iteration allocation supplies the
  destination half of that proof and not the resource half. Recorded, not
  designed.

### 2.9 Why `loop_stmt` is admitted, and why that does not break the invariant

C argued that extending the rule family to `loop` "would make permission depend
on a derived fact about two indices and would break that invariant on the day
it landed", citing `loop_permission.rs:73-79` (verified — the module doc does
say the quantification over iterations is structural, from [FN-1]'s
unit-increment recurrence over a compiler-owned binder).

That is true for an *index-range split*, which must recover a trip count and an
induction variable to divide the space. It is false for a pipeline, whose unit
is **the iteration**, which the statement graph already gives. The staged
judgment asks nothing about indices: it asks about the cut, the exits, the
loans, and the dispositions. `Survey`'s existing machinery answers all of them
for a `loop_stmt` body as readily as for a `CountedRange` body; the only change
is that `collect` (`loop_permission.rs:301`) must match both forms.

**On fact-independence.** Stage 1's staged permission reads **no** entailment
fact, so `permission.rs`'s and `loop_permission.rs`'s invariant stands
unchanged. Stage 2's iteration-privacy judgment **is** a fact consumer, and it
therefore does not go into `analyze_permission`: it lives in its own module,
runs after entailment, and is an *actualization* choice, not a permission. The
invariant it must satisfy is the weaker, correct one for a fact consumer, and
it is AGENTS.md's own: *an optimizer fact may improve an accepted program but
may not change acceptance or claim execution.* With facts degraded, every
summary is `All`/`Empty`, nothing is privatized, and every program compiles and
publishes the same bytes, more slowly. **That must be pinned by a test, not
asserted in a comment.** (C criticised this move and then made it itself, in
its own Edit 3 and §2.5. A was honest about it. The honest version is here.)

### 2.10 The host-resource hole all three designs missed

None of the three designs addressed this, and it can turn a correct program's
`Ok` into an `Err`.

In `many_files_narrow.wf` the `ReadFile` handle is closed at the end of
`region 'h`, so source order holds exactly one descriptor at a time. K-deep
pipelining holds K. [SYS-10] (`spec/kernel-spec.md:2572`) is explicit that this
is not an implementation resource:

> "Reserving it promises no native descriptor, handle-table entry, kernel
> memory, or host quota: host exhaustion remains the ordinary
> `ResourceExhausted` member of the open operation's typed `IoError` result."

And [PAR-1]/[PAR-2] excuse only "Exhaustion of the execution resources an
implementation spends on overlapping" (spec:2020, 2065) — descriptors consumed
by the program's own `open_file` calls are not that. So under a tight
descriptor limit the schedule can turn `Ok` into `Err(ResourceExhausted)`; the
narrow program's empty `Err` arm drops that file and the published checksum
changes, falsifying the identity clause.

**Resolution — retire and retry, in the runtime, below the language.** When an
open submitted while more than one slot is outstanding completes with
`ResourceExhausted`, the adapter does **not** publish that terminal outcome. It
completes every older slot in index order (which runs their compiler-derived
closes and returns their descriptors), reducing the loop's held-descriptor
count to the source-order footprint of one, and then performs the host attempt
again. Only the second attempt's outcome is published. If it also fails, that
is the outcome source-order execution produces, and the program sees it.

This is sound and has precedent: `wf_bridge_submit_file` (`bridge.c:435`)
already waits for capacity and retries internally, and the language sees one
`open_file` call with one outcome however many host attempts formed it. The
consumed `FilePermit` is not re-consumed; the retry is a host attempt, not a
language call. Cost is paid only on the exhausted path, so nothing on the
correct path changes — T3 holds.

One normative sentence distinguishes the two resource classes (§4.1). Without
it, an implementer reading [PAR-2]'s existing exhaustion clause could believe
descriptors are excused. They are not.

---

## 3. Lowering and runtime

### 3.1 The window K — who chooses it

**The runtime chooses it, once per loop entry, and the writer never sees it.**
The precedent to copy exactly is `wf__par_split_budget(span, weight)`
(`compiler/src/backend/par_runtime.c:943`, emitted at
`compiler/src/backend/emitter/parallel.rs:554`), whose module doc states the
discipline: "asked once per loop entry, never per iteration."

```c
uint64_t wf__completion_window(uint64_t span, uint64_t slot_bytes, uint64_t ceiling);
```

- **The compiler** supplies `slot_bytes` (computed statically from the
  privatized places' lengths, the slot's saved scalars, the staged path bytes,
  and the tokens) and a `ceiling` from storage cost — a static, uniform IR cost,
  the same discipline as `assign_weights`
  (`compiler/src/lowering/builder/split.rs:779`), which "reads no name,
  signature, or source shape". A loop whose slot storage alone exceeds the
  budget **declines at compile time, loudly**, exactly as `split.rs`'s
  `Decline` reports a lane frame that does not fit `LANE_FRAME_BYTES`
  (`compiler/src/lowering.rs:1163`). A loop privatizing a 16 MiB buffer gets
  K = 1 and a reported decline, not 512 MiB of invisible memory.
- **The runtime** answers from its own capacity: `WF_BRIDGE_OPERATION_CAPACITY`
  and `WF_BRIDGE_SLOT_COUNT` are 64 (`bridge.c:40-41`), the Linux ring is sized
  to the same 64 entries, `WF_BRIDGE_MAX_HELPERS` is 8 on Darwin
  (`bridge.c:43`), `WF_COMPLETION_RESULT_CAPACITY` is 256
  (`completion/contract.h:30`), and a byte budget divides by `slot_bytes`.
- **K = 1 is always a legal answer** and reproduces the sequential program
  exactly, so the query can never make a program fail. `WF_WORKERS=0` and
  `--no-overlap` keep their meaning: no pipeline, no ring, so the S line stays
  an honest control.

There is no environment variable, attribute, or source spelling for K.
`WF_IO_HELPERS` remains a target-policy knob for the helper pool, not a
language surface, exactly as today.

For `many_files_narrow.wf`: `slot_bytes` = 65,536 (`data`) + 16 (staged path)
+ ~48 (`index`, `handle`, `produced`, `digest`) + tokens ≈ 65.7 KiB. K = 32 is
2.1 MiB of runtime storage; K = 8 is 526 KiB.

### 3.2 Private storage per in-flight iteration

| what | where | size at K = 32 |
|---|---|---|
| ring slots for `data` (stage 1) or private copies (stage 2) | one heap allocation at loop entry | 2 MiB |
| ring slots for `name` (stage 1) | one heap allocation at loop entry | 512 B |
| staged path bytes, per slot | in the slot record | 32 x (component_limit+1) |
| slot scalars (`index`, `handle`, `produced`, `digest`, stage) | one `alloca [KMAX x slot]` in the entry block | ~40 B x 64 = 2.5 KiB stack |
| completion tokens, result slots, raw value/error | one `alloca [KMAX x …]` in the entry block | tens of bytes x 64 |

It is **not** language-visible heap: it does not enter `allocates(heap)`, for
the same reason a lane frame and a completion record do not. Under stage 2 slot
0 may alias the source's own buffer, so the marginal cost is
`(K-1) * slot_bytes`.

The private copies are **not** initialized from the shared buffer and **not**
chained. The coverage proof of §2.6 is exactly the statement that their initial
contents are never observed. Chaining (copy slot j-1 into slot j) is the
sound-without-proof alternative and costs 8,192 x 64 KiB = 512 MiB of memcpy
while reserializing the pipeline; it is rejected.

### 3.3 The ring and the restore (stage 1)

`buffer_new` emits `malloc` plus a full element-fill loop
(`compiler/src/backend/emitter/buffer.rs:45-70`) and release emits `free`. A
naive "just allocate inside the loop" is therefore **worse** than today:
8,192 x (malloc + 64 KiB fill + free) is 512 MiB of memset, which at container
memory bandwidth is larger than the entire remaining Linux gap. The ring makes
per-iteration allocation free:

> **Slot invariant.** A released ring slot holds the value its `buffer_new`
> constructed, over its whole capacity.

So the ring is a change of the **release action**, not of release *placement*.
It rides [FN-1]'s existing guarantee that every edge leaving an entered
`for_stmt` body normally carries exactly once every compiler-derived drop and
release (`spec/kernel-spec.md:656`). Instead of `free`, that edge emits a
restore:

- **Tier 1 — contract-bounded.** Every write into the slot inside B is one
  [SYS-8] operation. Restore exactly the union of the extents [SYS-8] fixes:
  `memset(slot, v, produced - start)` where `produced` is the value the
  `ReadBytes` arm already binds, nothing on `ReadEnd` and `ReadFailed`. **Reads
  no entailment fact**, so facts-off compilation is bit-identical.
- **Tier 2 — capacity restore.** Any other write into the slot (a user call
  with `writes(slot)` and no derivable extent, an element `set`, an unresolved
  write): restore the whole capacity. `name` is tier 2 at 16 bytes. Tier 2 is
  the **default**; tier 1 is the narrow optimization.
- **Tier 3 — decline.** Construction arguments not loop-invariant, or the value
  escapes the iteration: fall back to per-iteration `malloc`/`free`. The
  pipeline still applies.

**Where the restore is emitted, exactly.** B's document states two different
placements — "at the release site" in one paragraph and "per outcome arm" in
another — and the arm binder `produced` is out of scope at the
compiler-derived release. Settle it: **the restore is one action on the
compiler-derived release edge**, and its extent is a slot-record field written
by each outcome arm (0 on `ReadEnd`/`ReadFailed`, `produced - start` on
`ReadBytes`) and phi'd through the arm merge. One placement, one action, one
value, and the value's provenance is a binding the program already has.

Cost for the benchmark: 68 MiB of restore total — the same bytes `fold_bytes`
has just read, hot in L1/L2, on the same lane. Against 512 MiB of cold fill for
naive per-iteration allocation, that is the difference between free and
unusable. It is still real: it is CPU the privatized stage-2 form does not pay,
and it is one of the reasons stage 2 is worth building rather than declaring
stage 1 sufficient.

### 3.4 In-order commit, out-of-order harvest, and the emitter change

**What the tree does now, after park on miss (batch 2, `PARK-ON-MISS.md`).**
The three paragraphs of this section that named the shipped emitter were
written before that batch and are corrected here rather than left standing.
`emit_all_completion_joins`
(`compiler/src/backend/emitter/completion.rs:430`) is still called first thing
in `emit_terminator` (`compiler/src/backend/emitter.rs:1824`), so every
outstanding target operation is still joined before any block ends, including a
loop back-edge, and it still exempts `HandedOut::Compute`. What changed is
underneath it. An overlap group's join site now runs one order for both kinds
of member, `compute_join_order` (`emitter/parallel.rs:670`, design §4): the
group's compute members newest first, because the deque is Chase-Lev and the
newest entry is the one the owner can reach, and its completion members exactly
where they were published, because a completion member holds no deque entry and
the deque constrains it nowhere. `emit_overlap_joins`, `overlap_join_tail` and
`block_exit_label` all consume that one function. The completion record is no
longer a token into a runtime pool: it is an opaque block of the submitting
frame, reserved by the wrapper that submits, found by the runtime at its own
address (design §5), and a join that misses parks the joining stack instead of
holding the thread. The round barrier this section is about is therefore still
`emit_all_completion_joins`, and it is still one line; the cost of the join it
performs is now a park and not a blocked thread.

That one line is the complete explanation of the round barrier. Three changes:

1. `emit_terminator` joins all outstanding operations **except** those owned by
   a live pipeline slot of the enclosing loop. The back-edge is then legal with
   work in flight. `emit_completion_dependencies` already joins exactly the
   named prior operations and leaves everything else in flight — that
   per-token behaviour is what the pipeline needs and it is already correct.
   Note also that `emit_all_completion_joins` already exempts
   `HandedOut::Compute`, so the fold hand-out is untouched by it.
2. The pipeline emits its own driver and retirement blocks: look at each busy
   slot's own record, take the ones already published, and otherwise join the
   oldest through `wf__completion_file_join` (`bridge.h:116`); run that slot's
   next stage; commit finished slots in index order. (The non-blocking
   `wf__completion_file_take` this step was written against is gone with the
   slot pool: after park on miss there is one join, it reads the record in the
   frame, and a miss parks rather than blocks, so "try, then fall back to a
   blocking join" is now "join, and pay a park if it misses".)
3. The loop's normal exit and every exit edge from P emit a **drain**: retire
   every busy slot in index order — all of it work the sequential execution
   performs — apply their tails so `sum` and `bytes` take their source-order
   values, run their compiler-derived releases, then free the ring.

**No stackless generalization is required.** This is the single largest cost
saving in the design and it must be stated plainly to anyone who assumes
otherwise. The stackless continuation lowering it argued against no longer
exists: `emitter/stackless.rs` and `tests/stackless.rs` were deleted in batch
2's slice 2, and what replaced them is the stack park of `PARK-ON-MISS.md` §5 —
a join that misses parks its own stack on a pool stack of a fixed reservation
and switches, which is one save and restore of the callee-saved registers and
the stack pointer and no emitter transform at all. The judgment this paragraph
recorded stands and was reached from the other side: `StacklessPlan::build`
admitted only a single block, an empty `overlaps()`, exactly one may-suspend
call and a returning terminator, the whole 757-line file was shaped by those
four refusals, and the estimates to lift them ran from ~800 to 2,000+ lines
plus a large test surface for a coroutine transform emitted as LLVM *text*.
**The pipeline does not need any of it**: with K slots outstanding the owner
lane has nothing else to run, so its join misses and parks, which costs one
park and no lowering.

What replaces it is smaller and has an in-tree precedent. E is cut at each
subsequent suspension into **stages**, and each stage is a straight-line region
entered and left through the slot record; values live across a stage boundary
are spilled to that record. This is **state-machine outlining**, structurally
what `split.rs` already does when it outlines a chunk (`split.rs:183`), not a
continuation transform: every stage runs on the owner lane, there is no stack
switch, no cross-thread resume, and the persisted state is a fixed record of
four or five scalars. Budget it as such (§6.1) and do not let anyone
re-introduce a continuation transform into the critical path; the suspension
the design once wanted from one is the core's stack park, which is already
there.

### 3.5 The compute lane — and a claim that has no thread

The per-iteration compute must come off the owner lane, because that is the
whole of `N.pool2`'s advantage: `RESULTS.md` says it "folds each file's
checksum on the worker that read it, which is compute parallelism the Whitefoot
source cannot express."

**Use the par pool, not the writer scheduler.** A's Claim 2 routes a ready frame
to another lane through
`runtime.c:652 -> writer_scheduler.c:123 -> par_runtime.c:286-287`. Every
anchor is real, but `wf_bridge_helper_policy`
(`compiler/src/backend/completion/bridge.c:106-121`) sets `*initial = 0u` and
`*cap = 0u` whenever `wf_bridge_linux_ready != 0` — **zero helper threads on
Linux with io_uring, by deliberate batch-0084 policy, with the measurement in
the comment.** The only callers of `wf__writer_scheduler_help_once` are
`par_runtime.c:535,565,801` (a par worker) and
`bridge.c:1322` inside `wf__writer_run_root` (the owner thread itself). With no
helper and no par worker there is no thread for a ready frame to migrate to, so
on Linux/io_uring A's fold stays on the issuing lane and Claim 2 has no
mechanism. Both judges found this independently; it is the sharpest reason A
cannot be built as written.

The mechanism that does have threads is the existing compute hand-out:
`wf__par_claim` / `wf__par_publish` / `wf__par_join` / `wf__par_release`
(`compiler/src/backend/emitter/parallel.rs:106,430,443,612`) and
`FunctionEmitter::emit_handed_out_call` (`parallel.rs:386`). When slot i's read
retires, `fold_bytes(&data_i, produced_i, 0)` — a pure call whose only argument
places are the slot's — is handed out with the slot as the frame, and the
restore rides the same frame so the memset happens on the lane that just read
those bytes. `wf__par_claim` returning null (no free lane) runs the fold inline;
that decline path exists. On the 2-CPU Linux container `wf__par_lanes_once()`
is 2, so there is exactly one compute lane, which is precisely the shape
`N.pool2` has.

Several folds may be outstanding, one per slot. The owner joins and commits
them **in index order**, so no associativity is used and no combination tree is
built.

### 3.6 Two latent bugs the barrier is currently hiding

Whoever removes `emit_all_completion_joins` from the back-edge must fix these
first. Only design A found them; they are correctness prerequisites for every
design in the set, and each is a silent wrong-path defect rather than a compile
error.

1. **The adapter retains the caller's path pointer.**
   `bridge.c:722` stores `request.operation.open_at.path = path;` and the Linux
   native path passes the same pointer through
   (`wf_bridge_submit_linux_open_at(directory, path, …)`, `bridge.c:704`), where
   the SQE keeps it to completion
   (`submission->addr = (uint64_t)(uintptr_t)entry->request.buffer.path`,
   `linux_io_uring.c:495`, `IORING_OP_OPENAT`). The staged bytes must be copied
   into the operation record's own storage.
2. **`%component` is one static buffer per call site.**
   `compiler/src/backend/emitter/system.rs:1952` emits
   `%component = alloca [slot x i8]` inside an `alwaysinline` wrapper, used at
   `:2059` and `:2089`; after inlining that is one entry-block buffer shared by
   every iteration. `compiler/src/backend/emitter/completion.rs:463` allocates a
   second one the same way. With K opens in flight, iteration i+1's memcpy
   overwrites the buffer iteration i's `openat` is still reading.
   All of `completion.rs`'s per-site `entry_slot` storage — token, result slot,
   raw value, raw error, open outcome, position, component
   (`completion.rs:211-214, 393-397, 463, 552-556`) — must become slot-indexed
   arrays.

Fixing (1) and (2) is also what makes §4.3's `loan-released(name)` at
`begin_submit` true rather than aspirational, which is what admits `name` as a
serialized place in stage 2.

### 3.7 Early exit, and a false claim, with operations in flight

- **Typed exit from P(i)**: retire slots j < i in index order, apply their
  tails, run their releases, free the ring, then take the edge. Every
  observable the sequential execution would produce before that point is
  produced, and none after it. No slot j >= i exists.
- **Loop-normal exit**: the same drain, then the ring free.
- **A false `claim`**: [PAR-1]'s erroneous-execution clauses apply verbatim —
  one complete [DIAG-3] record, abort without unwinding and without language
  cleanup. In-flight operations are abandoned by process abort. **Under T3 the
  correct path pays nothing**: no latch is read, no slot is quiesced, no
  ordering is imposed to make the defect reproducible. `WF_WORKERS=0` remains
  the deterministic world. Expect every reviewer to propose draining before a
  trap or latching a "some iteration failed" flag; both are the shape T3 exists
  to refuse, and batch 0078 already deleted one instance of it.

### 3.8 Why this does NOT compose with the [PAR-2] range split

Both A (§3.4) and C (§3.5) propose nesting the range split outside the
pipeline so each lane runs its own pipeline over its own subrange. **That is
unsound and must not be built.** `split_counted_range`
(`compiler/src/lowering/builder/split.rs:183`) makes each lane run the whole
body, including `reserve_file(&uniq files)` — two lanes holding two
simultaneously live exclusive loans on one enclosing place, which is the
[OWN-5] violation [PAR-1]'s loan column exists to refuse, and which C's own
serialized disposition forbids two sections earlier. B reached the correct
conclusion and its paragraph is carried here.

The two mechanisms therefore do not compete: the range split serves loops whose
bodies touch no enclosing unique state; the pipeline serves loops that do I/O;
a loop qualifying for both has no may-suspend call, and there the pipeline
declines. `assign_weights`, `identity_value`, `combine_values` and the
`LoopCombine` set are untouched. Compute parallelism for the pipeline comes
from §3.5's per-iteration hand-out, which needs no range split at all.

### 3.9 What the completion runtime needs beyond what it has

Everything is in `compiler/src/backend/completion/`.

1. **`wf__completion_window(span, slot_bytes, ceiling)`** — new, ~40 lines in
   `bridge.c`, answering from `WF_BRIDGE_OPERATION_CAPACITY`, the Linux ring's
   entry capacity, `WF_BRIDGE_MAX_HELPERS` on Darwin, and a byte budget. Plus a
   weak fallback in `COMPLETION_RUNTIME_FALLBACK`
   (`compiler/src/backend/emitter/completion.rs:41`) returning 1, so a link
   without the completion unit is sequential.
2. **Per-operation-record path storage** — §3.6 item 1. ~180 lines across
   `bridge.c`, the file adapter, and `linux_io_uring.c`, plus publishing
   `loan-released(name)` at `begin_submit`.
3. **Deferred io_uring doorbell** — `wf_linux_kick_locked` is called on every
   submit inside the submission lock (`linux_io_uring.c:653`). A
   deferred-doorbell mode plus a flush on the first join, ~150 lines. **Worth
   7-20 ms by the corrected arithmetic of §0.1, not 35-45.** Probe A first.
4. **Retire-and-retry on `ResourceExhausted`** — §2.10. ~80 lines: the adapter
   must be able to ask the pipeline to drain and then re-attempt one request
   before publishing its terminal outcome.
5. **Harvest-many** — optional. `wf__completion_file_take` already gives
   non-blocking per-token progress; a `take_any(tokens[], n, &which)` would let
   one driver visit drain several completions from one `io_uring_enter` return.
   Measure before writing it.
6. **Nothing for backpressure.** `wf__completion_file_pread_submit` returns 0
   when the operation was not handed to a target and the caller uses the direct
   path (`bridge.c:617`), and `wf_bridge_submit_file` waits for capacity and
   retries internally (`bridge.c:435`). A full ring degrades to a direct
   blocking call — a throughput cliff, not a correctness problem. Instrument
   `wf__completion_file_fallback_submissions` (`bridge.h:162`) in the bench run
   and set K below the runtime's capacity; do not design for it yet.
7. **Nothing on the protocol core.** The 34.9-35.6 ns/op round trip is 0.86 ms
   across this workload's ~24,600 operations. It is not the cost and must not
   be touched.
8. **A ring-off target policy knob, for Probe C only** — §5.3. There is no
   `getenv` in the completion sources except `WF_IO_HELPERS` (`bridge.c:90`)
   and the harness's `WF_REQUIRE_LINUX_IO_URING`; measuring "helpers instead of
   ring" needs one more, in the same class as `WF_IO_HELPERS` — a target-policy
   knob, never a language surface.

### 3.10 The match-scrutinee gap, which is a bug on its own

`IrBuilder::lower_statements` inserts into `call_results` only from the
`CheckedStatement::Let` arm, and only for `UserCall`/`SystemCall`
(`compiler/src/lowering/builder.rs:739-757`), so `completion_steps`
(`builder.rs:649-712`) cannot see `match open_file<…>(…) {`.
`many_files_narrow.wf` writes both of its system calls as match scrutinees;
`many_files_wide8.wf` writes them as `let` bindings. **That single syntactic
difference is part of why one program submits zero operations and the other
submits fourteen**, it has no semantic warrant, and it is a small fix worth
landing on its own, before anything else in this design (batch 0, §7).

---

## 4. Spec delta

Minimal: **one rule amended, one rule amended by three sentences, one rule
amended by two. No new rule, no grammar production, no keyword, no operation,
no type, no outcome, no writer-visible marker.** Rule count stays 137.

### 4.1 [PAR-2] — a second, staged permission

Keep [PAR-2]'s opening clause verbatim and name it the *counted permission*.
Add, after its final paragraph:

> This rule additionally defines a **staged permission**, over the body of any
> `for_stmt` or `loop_stmt` L, which admits a different overlap and requires
> none of the counted permission's accumulator, combination, or index
> conditions.
>
> Permission holds for the staged schedule exactly when all of the following
> hold, writing B for L's body and forming every written, read, and
> operand-read footprint and every loan of a statement of B exactly as [PAR-1]
> forms one.
>
> There is one program point c of B such that every statement of B either
> executes before c on every path through B or is reached only through c, and c
> is the argument evaluation and submission of the first `may-suspend` action of
> B in program order. Write P for the statements up to and including c and E for
> the rest.
>
> Every edge that leaves B — a `return_stmt`, a `give_stmt` delivering outside
> B, a `break_stmt` naming L or a loop enclosing L, and a `let_stmt` selecting
> `propagate_let_rhs` — occurs in P.
>
> Every borrow a `may-suspend` call of B retains past its own submission is on
> a place rooted in a binding B itself introduces, on a place this rule
> replicates, or on a place no footprint of B writes. Every exclusive loan a
> call of E holds is on a place rooted in a binding B itself introduces or on a
> place this rule replicates.
>
> Every place rooted in a binding declared outside L that a footprint of B
> reaches satisfies one of exactly three conditions, and a place satisfying none
> denies permission. Either no footprint of B writes it and every loan on it is
> shared; or every footprint element and every loan touching it belongs to a
> call in P and no loan on it is retained past c; or this rule replicates it.
>
> Every call and compiler-derived release in B has a complete target summary; a
> footprint element, loan, extent, or statement form the implementation does not
> resolve denies permission rather than granting it.
>
> Under the staged permission an implementation may execute the segment E of one
> iteration with overlapping execution against the segment P of any later
> iteration, and against the segment E of any other iteration. **The executions
> of P for the iterations taken in index order do not overlap one another, and
> no execution of P begins before the execution of P of every earlier iteration
> has completed.** Every write E performs to a place rooted outside B occurs in
> the order of the iterations that perform it. Under a permitted staged overlap,
> bindings and every Whitefoot state place equal the source-order result, on
> exactly the terms the counted permission states, including its
> erroneous-execution clauses.
>
> An implementation may replicate a place rooted outside L, giving each
> concurrently executing iteration its own storage of the same length, only when
> that place's element type is copy, when no statement L's continuation reaches
> reads it, and when on every path through B every byte of it a footprint of B
> reads was written by an earlier footprint of B on that path. The bytes one
> footprint reads and writes are exactly those its operation contract fixes for a
> system operation [SYS-8], those the callee's own summary fixes for a user call
> after [EFF-2] boundary projection, and the exact subscripted position for a
> direct element access; observing a place's length reads no byte of it. An
> extent the implementation does not resolve is the whole place for a read and
> empty for a write, and an underivable containment denies replication rather
> than granting it.
>
> When an execution of one iteration leaves L through an edge of P, the
> overlapped execution produces exactly the observables the source-order
> execution produces before that point and produces none after it; every
> operation of an earlier iteration still outstanding is completed and its
> segment E performed before that edge is taken.
>
> **The host resources a system operation of L creates are not execution
> resources an implementation spends on overlapping. An overlapped execution
> delivers for each operation of L an outcome that operation could deliver in the
> source-order execution at that point, so an implementation whose overlap holds
> more such resources at once than the source-order execution holds completes
> the earlier iterations and performs the operation again at the source-order
> resource footprint before delivering any outcome.**
>
> The number of operations an implementation keeps outstanding, the identity of
> the host thread that executes a segment, whether any overlap was performed at
> all, the storage an implementation gives a replicated place, and the storage an
> implementation reuses across iterations for a construction whose value the body
> releases without observing it, are not observable, and no rule of this
> specification is stated in terms of them. An implementation that overlaps
> nothing therefore conforms.

Three drafting notes for whoever transcribes this into `spec/kernel-spec.md`:

- The bolded prologue-serialization sentence is **load-bearing**: it is what
  admits `reserve_file(&uniq files)` without replicating a quota, and neither A
  nor B stated it. Do not drop it as redundant.
- B's original (P3)/(P4) said "every loan it holds" and "the written footprint
  of E overlaps neither footprint of P", which as written deny B's own program
  (`open_file` holds a shared loan on the enclosing `cwd`; P writes `name` and E
  reads it). The text above replaces both with the retained/exclusive
  distinction and the three-condition disposition sentence.
- The last clause says "releases without observing it", not B's "releases
  unread": in the ring's own target program the body *does* read `data`
  (`fold_bytes`); what it does not do is observe the value the release action
  sees.

### 4.2 [SYS-8] — the observed and defined byte ranges (three sentences)

To follow "Buffer and cursor disposition is exact." (`spec/kernel-spec.md:2530`):

> The bytes each range-bearing operation observes and defines in its buffer
> parameter are exactly these. `read_at` observes no byte of `destination` and,
> on `ReadBytes(next)`, defines exactly `[start, next)` from the transfer, so
> those bytes' post-call values do not depend on their pre-call values; on
> `ReadEnd` and on `ReadFailed` it defines none. `write_once` observes exactly
> `[start, end)` of `source` and defines no byte of it; `directory_next`
> observes no byte of `destination` and, on `ListBytes(next, entries)`, defines
> exactly `[start, next)`; `open_file` and `open_directory` observe exactly
> `[start, end)` of `name` and define no byte of it; `host_copy_bytes` and
> `host_copy_utf8` observe no byte of `destination` and, on `Ok(next)`, define
> exactly `[start, next)`. Observing a buffer's length observes no byte of it.

The existing text is an **upper bound on the change set** ("may have changed");
privatization needs a **lower bound** — that those bytes' post-call values do
not depend on their pre-call values — and the restore needs the same word. This
is the cheapest high-value sentence in the whole design; both `pread` and
`IORING_OP_READ` satisfy it.

### 4.3 [SYS-2] — the `name` borrow's release milestone (one sentence)

To follow "each borrow held for the call remains live until its own
`loan-released(path)` fact holds":

> `open_file`'s and `open_directory`'s `name` borrow is released before target
> transfer: forming the request copies the admitted `[start, end)` range into
> compiler-owned storage, and that copy is the operation's last access to the
> caller's buffer. Every other retained borrow of a `may-suspend` operation is
> released at `terminal`.

[SYS-2] already carries the structure — "one `loan-released(path)` fact for
every retained borrow … Keeping them distinct is required contract structure,
not a promise that later operations publish them together." This fills one entry
in a table that already exists. **It is false in the implementation until
§3.6's two bugs are fixed; land them in the same change.**

### 4.4 META-5 delta shape

> META-5 delta declaration: numbered rules +0/-0 (137 remain); grammar
> productions +0/-0 (75 remain); unique fixed lowercase grammar atoms +0/-0;
> writer operation spellings +0/-0; opaque system nominal spellings +0/-0;
> runtime-trap families +0/-0; entry forms +0/-0; contract block forms +0/-0;
> system operations and declaration records +0/-0 (203 remain); exception
> clauses +0/-0. [PAR-2] is amended to carry a second, staged permission over a
> cut of any `for_stmt` or `loop_stmt` body, whose conditions are formed
> entirely from [PAR-1]'s existing footprint and loan machinery and which
> requires no accumulator, combination tree, identity element, or index range;
> that permission additionally admits replicating a copy-element place under a
> byte-coverage condition, states that prologue executions do not overlap one
> another, distinguishes host resources a system operation creates from the
> execution resources an implementation spends on overlapping, and extends the
> non-observability clause to replicated storage and to storage reused across
> iterations for a construction the body releases without observing it. [SYS-8]
> is amended to name what each range-bearing operation observes and defines,
> which are entries in contract structure it already fixes and are stated rather
> than added. [SYS-2] is amended to name the `name` borrow's release milestone,
> one entry in a table it already carries. No writer-visible depth, window,
> queue, batch operation, task, future, callback, cancellation handle, or
> scheduling marker is added, and no ownership, effect, release, or trap rule
> changes. No accepted program becomes rejected: the permitted-overlap set only
> widens, so no conformance verdict moves.

Protected conformance: the staged permission needs its coverage annotation
prepared with the merge, in the same protected-class audit as [PAR-1]'s and
[PAR-2]'s.

### 4.5 The writer-visible forms this design refuses

Recorded once so nobody re-litigates it. This table is C's, and it is the part
of C that survives its own verdict.

| form | what it says | verdict |
|---|---|---|
| a per-iteration resource clause, `for @l i in a..b own { data: buffer<u8> = … }` | "this storage is per iteration" | **Refused, R3.** A `let` in the body says it already, and `Survey::is_iteration_own` already believes it. A second spelling of an existing fact. |
| an array of buffers with a proven-distinct index, `data[i % 8]` | "these destinations are distinct" | **Refused.** The writer states the `8` — a scheduling knob in ownership clothing. Strictly worse than privatization, which needs no source change. |
| a batch operation, `read_all(files, names, destinations)` | "do these N together" | **Refused** by W1 and by the owner's own ruling: a batch API the writer does not reach for is worthless. FIRST-PRINCIPLES §17.2's *single-resource* vector operation (one unique loan, several physical transfers) remains legitimate and is a different thing. |
| `independent for` / `par for` / an independence attribute | "trust me, iterations do not interfere" | **Refused, W3** if trusted (a writer-accessible escape from the checker); **refused, R1** if checked (it grants nothing the check already grants). |
| a sub-range shared view, `slice_of(&'r P, start, end)` | "this is the part I read" | **Near miss, kept out.** No writable target path may traverse a slice, so it cannot carry a *written* extent and cannot cover `name_at`. Revisit only if a call-site read-extent spelling is ever wanted. |
| a required extent on every buffer-typed effect path | "these are the bytes I touch" | **Refused here**, though C accepted it: it makes `name_at` fail to compile (§2.7), it costs a grammar production and a corpus-wide plus conformance-wide rewrite over several versions, and §2.6 derives the same fact with no source change. |

---

## 5. Expected result

### 5.1 Linux — the arithmetic, corrected

`RESULTS.md`, batch 0086, 2 CPUs, kernel 6.8 aarch64 container, medians of nine:

```text
line          median    user     sys     CPU     cores
N.direct       72.04   52.20   20.08   72.28    1.00
N.pool2        40.21   54.54   23.31   77.85    1.94   best native
N.uring32      82.46   49.47   53.20  102.67    1.24   best one-thread native
C.wide8       119.47   65.65   61.42  127.07    1.06   best Whitefoot today
C.narrow      346.65   86.07  151.25  237.32    0.68
S.narrow      345.95   87.66  148.51  236.17    0.68   the --no-overlap control
open+close budget for the whole workload: about 11 ms
```

Three effects have to land together, and only two of them are confident.

1. **Depth removes the barrier — confident.** `C.wide8` at width 8 with a join
   per round is 1.26x behind `N.uring8` and 1.45x behind `N.uring32`. Continuous
   depth at K = 32 removes exactly that gap. Alone this is bounded below by the
   CPU: ~119 -> ~125 ms of CPU on one core, i.e. **it buys almost nothing on its
   own**, which is why any "remove the barrier and we reach pool2" claim is
   refuted by its own table.
2. **The fold moves to the second core — confident in mechanism, unmeasured in
   size.** `RESULTS.md` attributes `C.wide8`'s 65.65 ms of user CPU to "its own
   fold and program logic". No run has separated the fold from the rest, so the
   split is **estimated, not measured**. Probe D (§5.5) separates it in one
   afternoon. If the fold is F of the 65.65, handing it out puts F on lane 1 and
   leaves 65.65 - F on the owner lane.
3. **Batched submission attacks the system time — worth 7-20 ms, not 35-45.**
   §0.1 shows the native ring is already batched and still spends 53.20 ms of
   sys, so the residual is per-SQE processing, not doorbell entries. Deferring
   the doorbell saves the syscall entry cost on ~24,576 calls.

Composed, with F estimated at 45-55 ms and sys at 45-55 ms after batching:

```text
  owner lane  ~= (65.65 - F) + sys'   =  11..21  +  45..55   =  56..76 ms
  compute lane ~= F                                          =  45..55 ms
  wall        ~= max of the two, plus imperfect overlap       =  65..85 ms
```

**Honest verdict on Linux.**

- The owner's own falsifier — the narrow program at or below `C.wide8`'s
  119.47 ms — is met with large margin. **Confident.**
- Beating `N.uring32` at 82.46 ms is **probable, not assured**: the predicted
  band straddles it.
- Reaching `N.pool2` at 40.21 ms is **not reachable by this mechanism** and the
  document will not claim it. To land at 40 ms the program must spend ~80 ms of
  CPU across two cores; it spends 127 ms today and this design removes perhaps
  15 of them. §5.3 names the only route that closes that gap.

### 5.2 macOS — host-limited, and the bar is at risk

```text
N.pool8       373.91   78.45 user  528.45 sys   best native
N.pool10      378.30                            (worse than 8: the host saturates)
C.wide8       545.50   97.10 user  682.54 sys   best Whitefoot today
S.wide8      1130.97                            its own sequential build
C.narrow     1183.83
```

One `openat` costs ~116 us on this host because of an endpoint-security stack,
so the workload is ~950 ms of serial open time and nothing else matters much.
The best native shape converts 1113 ms of serial work into 368 ms — 3.0x on
eight threads, so the security stack serializes most of the concurrency a pool
asks for. `C.wide8` gets 2.07x. `RESULTS.md` names what `C.wide8` still does
serially: "its own direct open, the eight folds, the eight releases, and the
join of all eight before the next round starts."

The pipeline removes three of those four. If continuous depth reaches the same
3.0x ceiling the pool reaches, the program lands at 1131/3.0 ≈ **377 ms**.
Depth on macOS is bounded by `WF_BRIDGE_MAX_HELPERS = 8`, which is exactly the
pool's width, and `N.pool10` at 378.30 is *worse* than `N.pool8`, so there is no
depth advantage available above 8 — only the removal of Whitefoot's own serial
tax.

**Stated plainly: the macOS bar as written — beat `N.pool8` — may not be
reachable by this mechanism, and the reason is the host, not the design.** The
prediction is **380-450 ms**: past `C.wide8`'s 545.50 with margin, a ~3x
improvement on `C.narrow`, and parity rather than a win against `N.pool8`. The
owner's own falsifier (≤ 545.50) is met comfortably. §8 asks the owner whether
parity is acceptable.

### 5.3 The only route to `N.pool2`, and it is a runtime policy question

On this container a ring operation costs ~2.5 us of CPU and a blocking syscall
costs ~0.82 us (§0.1). `N.pool2` wins by doing blocking syscalls on two
threads. The completion model can express exactly that shape without a language
change: it is the helper pool. But `wf_bridge_helper_policy`
(`bridge.c:106-121`) starts **zero** helpers whenever the Linux ring is ready,
and the comment records why — "115 ms at zero helpers against 171 ms at one,
two, or four" — measured **on the four-wide, barrier-bound program**, where
depth was scarce and a helper handoff bought nothing.

That premise changes under this design. With a K-deep pipeline supplying
continuous depth and a compute lane taking the fold, the measurement that
justified the policy no longer describes the workload. **Probe C** (§5.5) tests
"helpers instead of the ring" with the pipeline in hand. If it wins, the Linux
answer to `N.pool2` is a policy change in `bridge.c` — a target policy, never a
language surface — and not a language mechanism at all. If it loses, the honest
statement is that on this host the Linux pool bar is not reachable, and the bar
should be restated against Probe B's hand-written ceiling.

Either way this is measured, not designed, and it is why Probe B and Probe C
come before implementation.

### 5.4 The falsifier

**The measurement, exactly.** Two programs, the existing harness unchanged:

```sh
make -C research/experiments/io-completion-bench verify   # identical bytes on every line
make -C research/experiments/io-completion-bench bench    # macOS, medians of 15 after 2 warm-ups
make -C research/experiments/io-completion-bench linux    # Linux, medians of 9 after 2 warm-ups
```

Built under batch 0086's protocol: before and after built as two compilers from
the same sources, both sets of binaries run interleaved in one plan.

**F1 — free, before any measurement.** `whitefootc --par-ledger`
(`compiler/src/bin/whitefootc.rs:366`) on `many_files_narrow.wf`
**byte-unchanged** must print a granted staged verdict for `@scan` naming each
place and its disposition: `cwd` read-only, `files` serialized, `name`
serialized, `data` privatized, `sum`/`bytes` ordered writes, `index` carried
datum. A denial names its condition and its node. If the denial is the coverage
condition on `data`, §2.6's derivation is wrong about what the entailment state
holds at the `fold_bytes` call. This costs nothing and fires first.

**F2 — the submission counter.** `wf__completion_file_submissions()`
(`bridge.h:161`) is **0** today for the narrow program and must be ~24,576 after
(one open, one read, one close per file). If it is 0, nothing actualized
regardless of what the ledger said, and the gap is in the lowering, not the
judgment. This separates a judgment failure from an actualization failure.

**F3 — the wall clock, and this is the owner's bar.**

```text
  REQUIRED  Linux   C.narrow.after  <=  119.47 ms   (C.wide8's own median)
  REQUIRED  macOS   C.narrow.after  <=  545.50 ms   (C.wide8's own median)
  claimed   Linux   C.narrow.after  <=   82.46 ms   (N.uring32)     probable
  at risk   Linux   C.narrow.after  <=   40.21 ms   (N.pool2)       §5.3
  at risk   macOS   C.narrow.after  <=  373.91 ms   (N.pool8)       §5.2
  control   both    S.narrow.after  ==  S.narrow.before within spread
  control   both    published checksum bytes identical on every recorded run
```

**with no change whatsoever to `many_files_narrow.wf`.** The two REQUIRED lines
are the falsifier the owner named: the natural one-file-at-a-time loop must
match the hand-written eight-way program. If either fails, the mechanism did not
deliver W1 and no argument about depth rescues it.

**F4 — the leak test.** Rebuild `many_files_wide8.wf` and confirm it is
**unchanged**. Its `Err` arms return, so the staged permission denies it by the
exit condition, and it must keep exactly the [PAR-1] window overlap it has
today. A regression there means the new fact-consuming judgment leaked into
`analyze_permission`.

**F5 — the discriminating pair.** `many_files_narrow.wf` and the same file with
`produced` replaced by `65536_u64` at the `fold_bytes` call (§2.3), both
maintained, both with checked published checksums, one on the permitted side and
one on the denied side. A privatization test that passes because privatization
never fired proves nothing.

**F6 — facts-off identity.** Compile the narrow program with the entailment
state degraded; acceptance must not move and the published bytes must not move.
Pinned by a test, not by a comment (§2.9).

### 5.5 Five probes to run before writing a line of the compiler

Each costs about an afternoon and each can change the plan.

- **Probe A — the doorbell.** Patch `bridge.c`/`linux_io_uring.c` to defer the
  io_uring kick and flush on first join. Run `C.wide8` byte-unchanged and read
  the `sys` column. §0.1 predicts a fall from 61.42 ms to 45-55, not to 25. If
  it does not move at all, drop item 3 of §5.1 and the Linux prediction becomes
  75-95 ms.
- **Probe B — the ceiling.** Hand-write the C program this design is chasing:
  depth 32 continuously in flight on one thread, folds on a second, on the same
  tree. If that does not beat `N.pool2`, **no Whitefoot lowering will**, and the
  Linux bar should be restated against it before implementation starts.
- **Probe C — helpers instead of the ring.** Add the target-policy knob of
  §3.9 item 8, run `C.wide8` with the ring off and helpers at 2, and read the
  wall and `sys` columns. This is the only measurement that speaks to §5.3, and
  it decides whether the `N.pool2` bar is a policy change or unreachable.
- **Probe D — separate the fold.** Time the narrow and wide8 programs with
  `fold_bytes` returning `seed` unchanged. This is the only way to know how much
  of the 65.65 ms of user CPU moves to lane 1, and effect 2 of §5.1 is estimated
  without it.
- **Probe E — does the summary fall out?** Before committing to batch 3,
  transcribe `fold_bytes`'s `loop @fold` with its manual index and two `break`s
  into `for @fold at in 0_u64..produced`, and check by hand that the derived
  read extent `[Z, produced)` falls out structurally from [FN-1]'s
  compiler-owned recurrence rather than from an immediately dominating guard.
  §2.6's derivation depends on `at < produced` being live at
  `deref(source)[at]` with no intervening [ENT-5] kill; if the counted form is
  the only spelling the analysis can summarize, that is a W1 exposure worth
  knowing about before 800 lines are written, and it must be *recorded* rather
  than papered over — a second natural spelling of the same fold that fails to
  privatize is a W1 defect the moment someone writes it.

---

## 6. Cost

### 6.1 Implementation size by component

| component | file | lines |
|---|---|---|
| match-scrutinee call results (batch 0, standalone) | `compiler/src/lowering/builder.rs` | ~80 |
| per-operation-record path storage; `loan-released(name)` at `begin_submit` | `completion/bridge.c`, file adapter, `linux_io_uring.c` | ~180 |
| slot-indexed completion storage (token, result, raw value/error, component) | `backend/emitter/completion.rs`, `backend/emitter/system.rs` | ~200 |
| staged judgment ((P1)-(P7), the four dispositions), reusing `Survey`/`Footprint`; `loop_stmt` admission | `semantic/loop_permission.rs` | ~500 |
| judgment tests | `semantic/tests/loop_permission.rs` | ~350 |
| ledger `stage` line and disposition report + tests | `semantic/permission_ledger.rs` | ~100 |
| pipeline IR: slots, ring, stage outlining, deferred joins, driver loop, fold hand-out, in-order commit | new `lowering/builder/pipeline.rs` | ~850 |
| back-edge-tolerant joins, retirement and drain blocks | `backend/emitter/completion.rs`, `backend/emitter.rs` | ~350 |
| ring construction and the tiered restore | `backend/emitter/buffer.rs`, `cleanup.rs` | ~200 |
| window query + weak fallback | `completion/bridge.c`, `emitter/completion.rs` | ~60 |
| deferred io_uring doorbell | `completion/linux_io_uring.c`, `bridge.c` | ~150 |
| retire-and-retry on `ResourceExhausted` | `completion/bridge.c` + adapter | ~80 |
| **stage 2:** derived byte-range summaries (`may_read`/`must_write` interval sets, call-graph fixpoint with SCC -> `All`/`Empty`, the [SYS-8] table, [EFF-2] substitution) | new `semantic/access_range.rs` | ~800 + ~300 tests |
| **stage 2:** privatization admissibility and slot allocation | `semantic/`, `lowering/builder/pipeline.rs` | ~250 |
| backend tests (shape, restore extents, drain on exit, `--no-overlap` parity, slot storage) | `compiler/src/backend/tests/` | ~600 |
| conformance cases and verdicts | `conformance/` | ~250 |
| bench programs, README, RESULTS, `docs/patterns.md`, `docs/done/` | `research/`, `docs/` | ~300 |
| **total** | | **~5,600** |

Of which **~3,400 is stage 1** (through the ring and the first bar) and
**~1,400 is stage 2** (the derived analysis that reaches the byte-unchanged
program), with the rest shared prerequisites. Comparable to batch 0078, which
built the loop judgment and the range splitter together, spread over three
merges. The estimate excludes `stackless.rs` entirely, and that exclusion is the
single most consequential engineering decision in the document (§3.4).

### 6.2 Risks, most dangerous first

1. **The Linux CPU floor.** `C.wide8` is already at 1.06 cores and a ring
   operation costs 3x a blocking syscall on this host. If Probe C does not win,
   the design beats `N.uring32` and does not reach `N.pool2`, and the charter's
   Linux bar is missed for a reason that is not the I/O model. *Mitigation:*
   Probes A-D before implementation; restate the bar against Probe B's ceiling.
2. **macOS is host-limited.** Predicted parity with `N.pool8`, not a win.
   *Mitigation:* say so now rather than after the work (§8, question 2).
3. **The restore's and the privatization's extents must be exactly right.** A
   missed extent is a silent wrong answer, not a crash. *Mitigation:* tier 2
   (full capacity) is the default and tier 1 the narrow exception; the interval
   set is not a prefix (§2.6); F5's discriminating pair is a required maintained
   program on both sides; the bench's byte check catches it end to end.
4. **The cut must be a real dominator query, not a statement-index heuristic**
   (§2.1). The natural body is nested four `region`/`match` levels deep. Getting
   (P1) wrong in the permissive direction breaks the exit rule.
5. **Stage 2 makes published bytes depend on the entailment fact state for the
   first time in this compiler.** That is a real widening of the trusted
   surface. *Mitigation:* it lives outside `analyze_permission`, its denial
   direction is total, and F6 pins facts-off identity. If the owner is not
   willing to take that step, stage 1 still ships and the W1 residual stands
   (§8, question 3).
6. **K slots at loop entry is a new exhaustion surface.** K x 64 KiB is 2 MiB;
   K x 1 MiB would not be. *Mitigation:* the byte budget in
   `wf__completion_window`, the compile-time `Decline` at large `slot_bytes`,
   and fail-down to K = 1. Needs a hostile test at a large `slot_bytes`.
7. **A full ring degrades to a direct blocking call** while K slots are
   outstanding — a latency cliff, not a correctness problem. *Mitigation:*
   instrument `wf__completion_file_fallback_submissions` in the bench run and
   set K below the runtime's capacity.
8. **The completion runtime and the compute-par runtime converge.**
   FIRST-PRINCIPLES §16 opens with "The completion runtime is separate from the
   pre-existing compute-par runtime", and this design has par lanes running
   folds handed out from a completion retirement. The plumbing already crosses
   that line (`par_runtime.c:286` externs `wf__writer_scheduler_help_once`), so
   the convergence is real today and only the volume changes — but it belongs in
   the design record, not in a review's discovery.
9. **T3 pressure.** Every reviewer will propose draining in-flight operations
   before a trap or latching a "some iteration failed" flag. Both are the shape
   T3 exists to refuse; batch 0078 already deleted one instance of it.
10. **The loan column's neighbourhood widens.** `loop_stmt` admission, the
    serialized disposition, and the exit relaxation all widen the neighbourhood
    batch 0081 audited. Its five closed holes — in particular the interposed
    statement and body-bound borrow cases — must be re-attacked against the
    staged judgment, because it admits body shapes [PAR-2] refuses outright.

### 6.3 Deliberately left out

- **Speculating past a typed exit**, and therefore cancellation. Needs
  FIRST-PRINCIPLES §8's closed-state proof extended to a target call whose
  *result* is a fresh resource. Costs the `wide8`-shaped program its open depth
  on macOS. Named in §2.8, not solved.
- **Stackless suspension inside loops.** Not needed (§3.4). The generalization
  in FIRST-PRINCIPLES §18.4.2 remains open for the case where a loop must yield
  to an unrelated writer frame.
- **Composing with the [PAR-2] range split.** Unsound as A and C proposed it
  (§3.8), and unnecessary: compute parallelism comes from the per-iteration
  hand-out.
- **Recombining the accumulator across lanes.** Not needed: the tail returns one
  own value and the owner commits it in index order. The existing [PAR-2]
  counted permission remains available for loops that qualify for it, and
  `LoopDenial::ManyAccumulators` is untouched.
- **A required extent column on effect paths** (§2.7), and the element-write
  map it would be a prerequisite for.
- **Cross-loop and nested-loop pipelining.** One loop, one window. "No rule of
  this specification joins two index ranges into one iteration space" is kept.
- **Any writer-visible depth, window, batch, vectored, or scatter/gather
  operation.** §4.5 records why, once, so it is not re-litigated.
- **`fold_bytes`'s per-byte cost.** Its `let readable = ilt(at, room); if
  readable { } else { break @fold; }` guard is a per-element branch the C
  baseline does not pay, and the checker cannot relate `produced` to
  `len(deref(source))` because the relation lives in the caller. That is a real
  W1 observation worth its own record and it is not this design's to fix;
  claiming otherwise would be dishonest about where the Linux CPU goes.
- **Windows IOCP.** The same shape applies through `windows_completion.c` and it
  is unmeasured. Do not claim it.
- **Cold storage, durable writes, network I/O.** The same exclusion the existing
  record carries.

---

## 7. Implementation sequence

Five merges. Each is independently mergeable, independently valuable, and
passes `make check` on its own. Sizes are the §6.1 rows.

**Batch 0 — prerequisites and probes. ~460 lines + four probes.**
Run Probes A, B, C, D (and E before batch 3) and write their numbers into `RESULTS.md` before deciding
anything downstream. Land the match-scrutinee `call_results` fix (§3.10, ~80
lines, a bug on its own merits), the per-operation-record path storage and the
`begin_submit` release milestone (~180), and slot-indexed completion storage
(~200). Nothing here changes a published byte or a permission verdict; all of it
is a correctness prerequisite for every later batch. **Exit condition:** the
probe numbers are recorded, and `C.wide8` is unchanged within spread.

**Batch 1 — the staged judgment. ~950 lines.**
`loop_permission.rs` gains the cut, (P1)-(P7), the four dispositions, and
`loop_stmt` admission; the ledger prints the disposition table; tests. No
lowering, no runtime, no measurement. **Exit condition:** F1 prints a granted
verdict for `many_files_loop.wf` (§1.2) and the expected denials for §2.3-2.5
and for `many_files_wide8.wf`. Ships alone as an honest, testable judgment even
if batch 2 slips.

**Batch 2 — the pipeline and the ring. ~2,000 lines.**
`lowering/builder/pipeline.rs`, the back-edge-tolerant joins, the driver loop
and retirement, the ring plus the tiered restore, `wf__completion_window`, the
deferred doorbell, retire-and-retry, the fold hand-out through `wf__par_claim`,
and the backend tests. **Exit condition:** F2 and F3 for `many_files_loop.wf`;
F4 for `many_files_wide8.wf`; the `--no-overlap` control unmoved. This is the
batch that produces the first performance number and the first spec merge
(§4.1, §4.2, §4.3, and the conformance evidence).

**Batch 3 — privatization by proof. ~1,400 lines.**
`semantic/access_range.rs` with interval-set summaries, the privatization
admissibility judgment, slot allocation for privatized places, and the
maintained discriminating pair. **Exit condition:** F1, F3, F5 and F6 for
`many_files_narrow.wf` **byte-unchanged**. This is the batch that answers the
owner's falsifier.

**Batch 4 — the record. ~550 lines.**
`RESULTS.md` program-level section, the bench README, `docs/patterns.md` gaining
the per-iteration-allocation writer form, `docs/done/0088-loop-pipeline.md`, and
the conformance cases and verdicts for each denial condition.

If the schedule must be cut, cut batch 3 and ship batches 0-2 with the W1
residual recorded honestly: the natural loop with per-iteration allocation is
fast, the hoisted form is refused with a diagnostic naming `&uniq 'd data`, and
the closing condition is written down. Do **not** cut batch 0; every later batch
is silently wrong without it.

---

## 8. Open questions that require the owner

These are decisions, not research. Each is stated with the evidence and with
what the answer changes.

1. **The Linux bar.** `N.pool2` at 40.21 ms is not reachable by this mechanism.
   §0.1 shows why: on this container a ring operation costs ~2.5 us of CPU
   against ~0.82 us for a blocking syscall, `C.wide8` already spends 127 ms of
   CPU, and this design removes perhaps 15 of them. The predicted landing is
   65-85 ms — past `N.uring32`, not near `N.pool2`. Three possible answers:
   (a) restate the bar as "beat every one-thread native shape, reach parity with
   the pool", (b) accept a runtime policy change if Probe C shows helpers beat
   the ring for this workload (§5.3), or (c) hold the bar and accept that it is
   met only after the fold's per-byte cost is separately attacked. **Which?**

2. **The macOS bar.** `N.pool8` at 373.91 ms is a 3.0x recovery from 1113 ms of
   serial open time on a host whose endpoint-security stack serializes most
   concurrency, and `N.pool10` is *worse* than `N.pool8`, so the host saturates
   at the helper cap this runtime already has. The predicted landing is
   380-450 ms — parity, not a win. **Is parity with the best native shape on a
   host-limited platform an acceptable outcome, or should the bar move?**

3. **Whether published bytes may depend on the entailment fact state.** Stage 2
   is the first place in this compiler where a *byte-correctness* argument
   consumes derived facts. Acceptance never moves, facts-off compilation
   publishes the same bytes (F6), and the judgment lives outside
   `analyze_permission` so `permission.rs`'s and `loop_permission.rs`'s
   fact-independence invariants stand. But the trusted surface genuinely widens.
   **Approve, or ship stage 1 only and keep the W1 residual on the books?**

4. **Amend [PAR-2], or add [PAR-3].** §4.1 amends [PAR-2] and keeps the rule
   count at 137, which is the minimal spec delta and what both judges rewarded.
   The honest cost is that [PAR-2] then carries two permissions that share no
   apparatus: the counted permission is about accumulators, combination trees
   and index ranges, and the staged permission is about a cut, dispositions and
   ordering. If rule-per-judgment legibility matters more than rule-count
   minimality, the identical text becomes **[PAR-3]** at +1 rule (138) and
   nothing else in this document changes. **Which does the owner want?**

5. **The named W1 residual: a body that returns on its first I/O error gets no
   pipeline** (§2.8). The rule must refuse it — with K in flight, iteration i's
   decision to return is taken after later iterations have performed `openat`s
   sequential execution never performs. On Linux this costs ~11 ms; on macOS,
   where an `openat` is 116 us, it costs nearly everything. Closing it needs
   FIRST-PRINCIPLES §8's closed-state proof extended to a target call whose
   result is a fresh resource. **Is this residual acceptable for now, or does it
   become the next project?**

6. **`many_files_loop.wf` as a second maintained program.** §1.2's
   per-iteration-allocation form is what batch 2 delivers against, and
   §1.3's byte-unchanged shipped program is what batch 3 delivers against.
   Keeping both is the only way to tell a judgment failure from an analysis
   failure. **Approve adding it to the bench corpus** (it is a new file in an
   existing home, wired to the existing harness, removed when the pipeline is
   retired).

---

## 9. Probe results (qemu container, provisional until the real-Linux CI run)

Appended in batch 0089. The five probes of §5.5 were run on 2026-08-27 against
this same base revision, and this section records what they measured. It does
not edit §0.1 or §5 in place: those sections are the prediction, this one is
the measurement, and keeping both visible is how the next reader can see which
predictions held. **Where the two disagree, this section is the fact and §0.1
is superseded.**

**Every Linux number below comes from the same qemu-virtualised Docker
container the batch-0086 table used — Ubuntu 24.04, kernel 6.8.0 aarch64, two
CPUs, 2 GiB, `--security-opt seccomp=unconfined`, tree on the container-local
filesystem. It is not a real Linux host, and by the owner's ruling it is not
performance evidence.** It is enough to show a direction and to kill an
arithmetic error; it is not enough to decide policy. The two decisions the
probes put on the table — whether the Linux ring should carry this workload at
all, and whether the direct depth-one path should bypass the ring — **wait for
the real-Linux CI numbers**, and nothing in batch 0 depends on either answer.
The macOS numbers in probe D are from the M4 development host and are labelled
as such.

One protocol note that bounds every comparison: **the container runs about
1.22x faster today than when the batch-0086 table was recorded** (`N.direct`
72.04 -> 57.8, `N.pool2` 40.21 -> 31.7, `N.uring32` 82.46 -> 66.3, `C.wide8`
119.47 -> 98.5). Every comparison below is therefore made against a reference
line measured in the same runner plan, never against the recorded table. Where
a batch-0086 equivalent is quoted it is marked as scaled by 1.22 and is never
used to decide anything.

### 9.1 Probe A — the doorbell

Deferring the `io_uring_enter` kick, measured with a target-policy knob in the
same class as `WF_IO_HELPERS`, on `C.wide8` with the source byte-unchanged
(medians of nine after two warm-ups, six interleaved passes):

```text
                    wall            sys            user
kick (= base)     98.5 ms        54.7 ms        50.7 ms
defer             84.1 ms        39.1 ms        46.7 ms
delta            -14.4 (-15%)   -15.6 (-29%)    -4.0 (-8%)
```

`io_uring_enter` calls fall **15,360 -> 2,048**. `C.narrow`, which has one
operation in flight and so nothing to batch, is unmoved: 299.7/301.5 with the
knob off against 299.5/305.6 with it on, in the two passes that carried the
line. That is the negative control. The design's §5.1 item 3 stands:
the doorbell is worth roughly what §0.1 predicted in total, and the wall falls
by the same amount as the sys column because in a barrier-bound single-lane
program the saved kernel CPU is all on the critical path.

### 9.2 Probe B — the ceiling

`probe/ceiling.c`, a hand-written C program written against the kernel io_uring
ABI directly, running exactly the shape this design proposes: depth 32
continuously in flight on one I/O lane, the fold on a second thread, slots
handed between them on a ready/free queue pair.

```text
N.pool2                              31.7 ms
B.ring2.d32   ring reads + fold thread   45.7 ms
B.ring2o.d32  ring opens too             45.6 ms
B.block2.d32  NO ring at all             45.1 ms
B.ring1.d32   ring reads, inline fold    68.0 ms   (control: reproduces N.uring32's 66.3)
```

Depth sweep for the two-lane ring shape: d8 = 50.7, d16 = 48.6, **d32 = 45.7**,
d64 = 52.8. Depth 32 is the optimum, as the design assumed.

**The hand-written ceiling of this design's own shape does not reach
`N.pool2`: 45.7 ms against 31.7 ms, a 1.44x gap, measured in the same runner
plan.** No Whitefoot lowering of that shape will beat it on this container.

**And io_uring is not the cause.** `B.block2` — the same producer/consumer
split with no ring at all, ordinary blocking `openat`/`pread`/`close` on the
I/O thread — lands at **45.1 ms**, indistinguishable from the ring shape's
45.7 and with the tightest spread in the whole record (44.6-45.8 across
eighteen runs). Swapping the transport moves the sys column by 8 ms and the
wall by nothing.

What binds at 45 ms is **the load balance of the fold**. `N.pool2` reaches
2.00 of 2 cores because each of its two threads does its own I/O *and* its own
folding; the two-lane shape reaches 1.65-1.69 because the whole fold sits on
one lane and the I/O lane finishes and waits. That reading is an inference from
the CPU columns, not a separately instrumented lane time, and the probe record
states it as one. It is a load-balance ceiling, not a transport ceiling, and no
amount of depth or doorbell batching moves it.

### 9.3 Probe C — helpers instead of the ring

The design's §5.3 hypothesis — that the bounded helper pool might beat the ring
for this workload — is **refused**. With the ring off and two helpers,
`C.wide8` runs 104.7 ms against the ring's 98.5: it trades 4 ms of system CPU
for 7 ms of user CPU and loses 6 ms of wall. One, four and default helper
counts all land in the same place.

The observation the design did not ask for is the large one:

```text
C.wide8   ring on, default          98.5 ms
C.wide8   ring off, ZERO helpers    66.0 ms      (-33%)
C.narrow  ring on, default         285.4 ms
C.narrow  ring off, ZERO helpers    62.1 ms      (-4.5x)
N.direct                            57.8 ms      (reference, same plan)
```

Zero helpers means the waiting scheduler runs the request itself, so the
program degrades to blocking calls made through the completion protocol. The
protocol tax is visible and small: 66.0 against `N.direct`'s 57.8 over 24,576
operations is 0.33 us per operation, consistent with the recorded ~35 ns
round trip plus bookkeeping.

**This is a Linux-container observation and must not be generalised.** On the
macOS host one `openat` costs 116 us and the same policy would destroy the
program. Any policy written from this has to be per-target, exactly as
`wf_bridge_helper_policy` already is — and it should be written from real-Linux
CI numbers, not from these.

### 9.4 Probe D — separate the fold

`fold_bytes`'s per-byte loop removed (its `let room = len(deref(source));` kept,
because a bare `return seed;` is refused by [EFF-2] as an over-declared
`reads(source)`), byte count unchanged, so only the fold is gone:

```text
Linux container   C.wide8    user 52.5 -> 10.1    the fold = 42.3 ms = 81% of user, 40% of wall
                  C.narrow   user 67.1 -> 22.5    the fold = 44.6 ms = 66% of user
macOS M4 host     C.wide8    user 66.4 -> 26.9    the fold = 39.5 ms = 59% of user, 7.9% of wall
                  C.narrow   user 50.1 ->  6.1    the fold = 44.0 ms = 88% of user, no wall effect
```

**The fold is 40-45 ms of user CPU on both hosts, in every program.** §5.1's
estimate of `F` at 45-55 ms can stop being an estimate; the measured value sits
at the bottom of that band. Re-running §5.1's composition with the measured
values predicts 49-55 ms, and probe B, which measured that exact shape by hand,
got 45.7 — two independent routes agreeing that `N.pool2` is out of reach for
this shape.

A secondary reading worth carrying: the non-fold user column separates the
platforms sharply (Linux 10.1 ms, macOS 26.9 ms), because on Linux the ring
does the work in the kernel and on macOS the helper pool does it in user
threads.

### 9.5 Probe E — does the read-extent summary fall out?

No compiler change was made. Four source spellings were compiled by the
unmodified base compiler, and the entailment implementation was read.

**Both fold spellings summarize.** `at < produced` is live at
`deref(source)[at]` in the counted (`for_stmt`) spelling and in the manual
`loop`-with-guard spelling alike, and the witness is the compiler's own
[ENT-6] discharge: with the room guard deleted and a `produced <= room`
precondition put in its place, the bounds obligation can be discharged *only*
by closing `at < produced` with it, and both spellings compile and publish the
identical checksum. The counted form gets its upper endpoint structurally from
[ENT-3.S11]'s body-entry facts; the manual form gets it from S1's exact
negation of the dominating guard. The lower endpoint `0 <= at` is free in both,
from the [ENT-4] closure's type-range seeding. §5.5's worry that only the
counted form would be summarizable is not borne out.

**The real W1 exposure is one call deeper.** `in_extent(at, produced)` — the
extent test moved into a `pure` helper, the most ordinary refactor an AI writer
performs — is **refused today**:

```text
whitefootc: Semantics/Source [OP-4]: UndischargedBoundsObligation {
  residual: "at < len(deref(source))", ... }
```

The relation crosses the call boundary as an opaque result and no bound reaches
the subscript. This is a **pre-existing** exposure, refused by today's [ENT-6]
before any privatization analysis exists, and the mechanism that would close it
already exists ([FN-1]'s ordered verified [FN-9] normal-result relation
templates, which `in_extent` publishes none of). Batch 3 should not be blocked
on it, but the design's own standard — "a second natural spelling of the same
fold that fails to privatize is a W1 defect the moment someone writes it" — is
met by this program and it is recorded rather than papered over.

### 9.6 Corrections to §0.1

§0.1 calls itself "the most important correction in the document". The probes'
`io_uring_enter` and per-opcode counters found five of its numbers wrong, all
because the operation counts were assumed rather than measured. Correcting them
makes the design's conclusions stronger, not weaker.

1. **`C.wide8` performs 15,360 ring operations, not 24,576, and zero closes.**
   `WF_IO_STATS enters=15360 sqes_handed_to_enter=15360 read=8192 write=0
   open=7168 close=0`. Seven of the eight opens per round reach the ring; the
   eighth is a direct `openat`. No close ever reaches the ring. The workload's
   host operations split 15,360 through io_uring and 1,024 direct opens plus
   8,192 blocking closes.
2. **`C.narrow` performs 8,192 ring operations, not zero.**
   `enters=8192 read=8192 open=0 close=0`. The narrow program submits every one
   of its reads to the ring and joins it immediately, which is why it costs
   285 ms against `N.direct`'s 57.8: a full synchronous ring round trip per
   file, overlapping nothing. §5.4's falsifier F2 — "`wf__completion_file_submissions()`
   is **0** today for the narrow program" — is therefore wrong as written and
   must be restated: the count is 8,192 today and the discriminating number is
   the **open** count, which is 0.
3. **A ring operation costs about 3.5 us of system CPU on this container, not
   the 2.5 us §0.1 computed.** Recomputing on the batch-0086 numbers: 9,216
   blocking operations at `N.direct`'s 0.82 us/op is 7.6 ms, leaving
   61.42 - 7.6 = 53.8 ms over 15,360 ring operations. §0.1's conclusion 2 —
   io_uring costs about 3x a blocking syscall in CPU here — is right and
   understated.
4. **The doorbell's saving is 1.17 us per avoided `io_uring_enter`, not
   0.3-0.8.** 15.6 ms of system time over 13,312 avoided calls. §0.1's band was
   computed over the wrong call count and landed on the right total.
5. **`--no-overlap` does not make the I/O calls direct on Linux.** The S line's
   binaries still report `enters=8192`: `--no-overlap` removes the overlap
   grouping, and the remaining direct call still goes through the completion
   bridge, which submits to the ring and joins. `S.narrow` and `S.wide8` are
   therefore not a sequential reference on Linux; the honest one is
   `WF_IO_RING=0 WF_IO_HELPERS=0`. The consequence for §5.4 is sharp: the
   REQUIRED Linux bar of 119.47 ms is met on this container today by an
   environment variable, so it no longer discriminates between a working
   pipeline and no pipeline, and the falsifier needs a bar the mechanism has to
   earn.

One §0.1 speculation is **neither confirmed nor killed**: the closing paragraph
attributes `N.uring32`'s 1.24 cores to kernel-side `io-wq` workers and lists it
as an upside probe A would settle. Probe A does not settle it. Today
`N.uring32` accounts 1.30 cores in the same plan, so the accounting artifact is
still there and nothing in the probe set attributes it. It remains speculation.

### 9.7 What is decided, and what is not

Decided by batch 0089 and implemented in it: nothing here. Batch 0 is
§3.6's two latent bugs and §3.10's match-scrutinee gap, and none of them
depends on a number in this section.

Not decided, and deliberately left open for the real-Linux CI run:

- **Should the Linux ring carry this workload at all?** Probe C says a
  ring-off, zero-helper policy is 33% faster on `C.wide8` and 4.5x faster on
  `C.narrow` *on this qemu container*. That is a target-policy question of
  exactly the kind `wf_bridge_helper_policy` already answers, and it must not
  be answered from a virtualised host.
- **Should the direct depth-one path bypass the ring?** The same probe, read
  the other way: `C.narrow`'s 8,192 submit-then-immediately-join round trips
  are the whole of its 285 ms, and a depth-one operation gains nothing from a
  ring. Whether the bridge should route such a call directly is the same
  question at a finer grain, and it waits on the same numbers.
- **The Linux bar itself.** §8 question 1 asked whether to hold `N.pool2` at
  40.21 ms. Probe B says no implementation of this shape reaches it on this
  container, and that the binding constraint is the fold's load balance rather
  than io_uring. The bar should be restated — against 45.7 ms measured here,
  or against whatever the same ceiling program measures on a real Linux host —
  before batch 2 spends 2,000 lines against it.
