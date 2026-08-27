# Batch 0091 — [PAR-3], the staged loop permission: a judgment with no lowering

Branch: `batch/0091-par3-judgment`, from main at `79b29665`.
Deliverables: the staged judgment and its ledger table, spec v0.38 activating
it as rule text, the seven maintained conformance shapes, the
`many_files_loop.wf` bench program, and this record.

## What this batch is, and what it deliberately is not

The design this implements (`LOOP-PIPELINE-DESIGN.md`, 2026-08-27) is five
merges. This is merge one: **the judgment only**. No lowering changes, no
runtime changes, no measurement. `many_files_loop.wf` compiles to exactly the
sequential module it compiled to before, and every published byte in the
repository is unchanged.

Shipping a judgment alone is worth a merge because it is separately falsifiable.
The design's own exit condition for this batch is a ledger reading, and the
ledger now reads it:

```text
many_files_loop.wf    for   permitted   6 places classified
many_files_narrow.wf  loop  denied      condition 3 at &'n name
many_files_wide8.wf   loop  denied      condition 2 at return exit_status(code: 11_u8);
many_files_wide.wf    loop  denied      condition 2 at return exit_status(code: 7_u8);
pipe_relay.wf         loop  denied      condition 2, a break
```

If the pipeline that a later batch builds turns out to be wrong, the failure is
now attributable: a wrong verdict here is a judgment defect, and a right verdict
with wrong bytes is a lowering defect. Batch 0081 made the same bet on the loan
column and it paid.

## The mechanism

[PAR-2]'s unit is an **index subrange**. That is why it needs a trip count, a
compiler-owned binder, an enumerated exactly-associative operation set, and a
combination tree, and why it admits only a `for_stmt`. C's design report
concluded from `loop_permission.rs:73-79` that extending the rule family to
`loop` "would break that invariant on the day it landed" — true for a range
split, false here.

[PAR-3]'s unit is **the iteration**, which the statement graph already gives. It
asks nothing about indices. It asks where the cut is, where the exits are, which
loans are retained, and what disposition each place carries. `loop_stmt` is
therefore admitted on exactly the same terms as `for_stmt`, with no trip count
and no induction variable recovered anywhere.

Write S for the first `may-suspend` call of the body B in program order, c for
the statement performing its argument evaluation and submission, P for the
statements up to and including c, and E for the rest. The schedule the rule
admits is: P(0), P(1), … in index order, never two at once; E(i)'s stages
executed after P(i+1..i+K) may already have run; E(i)'s writes to places rooted
outside the loop committed in the order of i.

Two sentences of that schedule are load-bearing and neither is derived:

- **Prologues never overlap one another.** This is what admits
  `reserve_file(&uniq files)` with no replication and no loan exemption. At
  every program point exactly one unique loan of the factory is live, so
  [OWN-5] is not relaxed — [SYS-10] already says the factory loan ends when the
  inline operation returns, and index-ordered prologues make those windows
  disjoint and in source order.
- **Every write E performs to storage rooted outside the body occurs in
  iteration order.** This is what admits `set sum = sum +wrap digest;` as an
  ordinary source-order write with no associativity, no identity element, and
  no combination tree. It is strictly more general than [PAR-2]'s accumulator
  apparatus: a non-associative fold, a float fold, and a `Result` route are all
  admitted here and none of them can ever be admitted there.
  `LoopDenial::ManyAccumulators` is untouched, and no [PAR-2] amendment was
  needed.

## The four dispositions, with the programs

Every place rooted outside the loop that a footprint of the body reaches gets
exactly one of four dispositions, and a place with none denies. The table is
what the ledger prints, and it is the reason a denial teaches something.

| disposition | condition | the program that shows it |
|---|---|---|
| **read-only** | no footprint of B writes it and every loan on it is shared | `cwd` in every bench program: `open_file`'s retained `&cwd` is shared, and two overlapping shared loans deny nothing |
| **serialized** | every footprint element and every loan touching it belongs to one segment alone, and no loan on it is retained past the cut | `files` in `many_files_loop.wf` (prologue, through `reserve_file`'s inline loan); `sum` and `bytes` (remainder, committed in index order) |
| **replicated** | copy element type, no continuation read, and every byte read was written earlier in the same iteration | `name` and `data` in `many_files_loop.wf`, which the body itself constructs |
| **denied** | everything else | `data` in `many_files_narrow.wf`; the enumeration cursor; storage reached on both sides of the cut |

Conditions 3 and 4 are stated over places, so their failures are the fourth
disposition rather than a fifth column: a place a `may-suspend` call retains a
borrow on and the body writes, or one a call of the remainder holds an exclusive
loan on, has no safe disposition.

The denials that had to be shown, each a maintained conformance program:

- **`accept-par3-staged-denied-hoisted-scratch.wf`** — the destination hoisted
  above the loop, folded at its full capacity rather than the prefix the
  transfer defined. After a short read the bytes beyond it are the previous
  iteration's, and the published checksum depends on them. A whole-place
  write-before-read rule would accept this and silently miscompile it.
- **`accept-par3-staged-denied-read-before-write.wf`** — the fold written before
  the transfer that fills the same storage. A true loop-carried dependence that
  no coverage proof can ever admit, kept beside the previous case so the two
  stay distinguishable when the byte-range analysis lands.
- **`accept-par3-staged-denied-carried-scratch-byte.wf`** — the marker byte a
  callee writes only on odd indices, so its written extent is not a fact of its
  signature. Executing prologues in index order would preserve that carried
  byte, but the remainder reads the same storage, so no single segment
  serializes it. Denied by condition 5.
- **`accept-par3-staged-denied-opaque-cursor.wf`** — an enclosing
  `DirectorySource` advanced through a retained exclusive borrow. Not
  read-only, not prologue-only, and an opaque system nominal has no copy
  element type, so no analysis can ever replicate it. The judgment knows this:
  the denial says "storage that carries one position cannot be held by two
  iterations at once" rather than telling the writer to allocate one per
  iteration, which would be wrong advice.
- **`accept-par3-staged-denied-exit-in-remainder.wf`** — the `wide8` shape, an
  early return after the submission.

## (P1) is a real dominator query

The condition is: there is one program point c such that every statement of B
either executes before c on every path through B or is reached only through c.
The natural body nests four `region`/`match` levels deep, and getting this wrong
in the permissive direction breaks the exit condition, so it is computed on the
body's own control-flow graph and never from statement indices.

`Flow::build` (`compiler/src/semantic/staged_permission.rs`) wires each
statement of B to its successors, with two sinks: `NORMAL_EXIT`, the back edge,
and `LEAVES`, every edge out of the loop or the function. Dominators run from
the entry; post-dominators run **with respect to the normal exit only**. That
second choice is what admits an early typed exit: a statement whose every
continuation leaves the loop reaches the normal exit on no path, so the
intersection over its successors is the whole node set and every node
post-dominates it, putting it in the prologue — where condition 2 then decides
whether it is admitted. A node the cut neither dominates nor post-dominates
denies with the statement cited.

One structural refusal sits ahead of the query: a submission written inside a
loop of B runs several times per iteration, so the body has no single cut and
the single-entry single-exit shape does not hold. That is a refusal, not an
analysis result.

## The ledger

Two line kinds, both anchored at the cut, because a `loop_stmt` carries no node
path of its own in the checked tree and the cut identifies the loop exactly:

```text
PAR stage       staged.wf:8  for   permitted   staged at open_file<'f, 'n>(…); 4 places classified
PAR place       staged.wf:8  serialized-P  &uniq 'f files  every footprint element and loan touching it belongs to the prologue, and prologues run in index order without overlapping
PAR place       staged.wf:8  read-only     &'f cwd  no footprint of the body writes it and every loan on it is shared
PAR place       staged.wf:8  serialized-E  set total = total +wrap 1_u64;  every footprint element and loan touching it belongs to the remainder, whose writes to storage rooted outside the loop commit in index order
PAR place       staged.wf:8  replicated    let name = buffer_new(16_u64, 97_u8);  iteration-own storage with copy elements, which an implementation may give each in-flight iteration its own of
```

A denial names the numbered condition, the place, and **one admitted writer
form**, and the form comes from the judgment itself so it cannot drift from the
condition that produced it:

```text
PAR stage  hoisted.wf:9  for   denied  condition 3: a may-suspend call retains a borrow
past its own submission on storage the body writes and the iteration does not introduce;
instead, allocate the scratch storage inside the loop body, so each iteration owns the
buffer it reads and writes, at &uniq 'd data
```

The table is printed for a **granted** loop too. A denial without a table says
only that a loop lost its pipeline; a granted loop's table is what a reader
checks a later change against. `docs/patterns.md` P15 teaches the form the
judgment grants, with the two companion rules — take every early exit in the
prologue, write the accumulator as an ordinary `set` — and points at
`--par-ledger` rather than asking the writer to guess.

A loop whose body performs no I/O gets **no** `stage` line at all: there is no
cut and no staged schedule to permit. Ledger volume grows exactly where the
judgment has something to say, and every counted loop in the corpus keeps
precisely the `PAR loop` line it had.

## The stage-one limitation, stated plainly

The replicated disposition here admits **only storage the body itself
constructs**. An enclosing scratch buffer that the body writes and reads is
denied, and the denial names it and points at the per-iteration form. The rule
text states the full condition — copy element type, no continuation read, and
every byte read written earlier on the same path — because that is the
permission, but discharging it for a hoisted place needs a derived byte-range
analysis over must-write and may-read interval sets, and that is a later batch.
Two consequences follow and neither is hidden:

- `many_files_narrow.wf`, byte-unchanged, is **denied**. The design's F5
  discriminating pair (`…-hoisted-scratch` against `…-read-before-write`) is
  registered and maintained, but stage one denies both members for the same
  reason, so the pair does not yet discriminate. It will when the analysis
  lands, and it is in the corpus now so that it can.
- The `name` borrow is denied for a second, separable reason: `open_file`
  retains `&name` to `loan-released(name)`, which [SYS-2] publishes at
  `terminal` today. Batch 0089 is landing the `begin_submit` release milestone
  in parallel; this batch does not depend on that branch and fails closed
  without it, reading every retained borrow as retained to `terminal`. When the
  milestone lands, `name` becomes serialized rather than denied and this
  judgment needs one condition-3 amendment, not a redesign.

## The W1 residual, recorded

**A body that returns on its first I/O error gets no pipeline.** The rule must
refuse it: with K iterations in flight, iteration i's decision to return is
taken after i+1..i+K-1 have already submitted opens that the source-order
execution never performs, and an `openat` is an externally observable state
transition that is not rolled back.

The cost is platform-shaped. On Linux an `openat` is ~0.85 us and the whole
open-plus-close budget is about 11 ms of a 119 ms program, so moving the cut to
the `read_at` site would lose almost nothing. On macOS, where one `openat` costs
~116 us because of an endpoint-security stack, it loses nearly everything.
`many_files_wide8.wf` is exactly this shape and is denied.

Closing it needs FIRST-PRINCIPLES §8's closed-state proof extended to a target
call whose *result* is a fresh resource; per-iteration allocation supplies the
destination half of that proof and not the resource half. This is recorded as a
named residual, not solved, and the owner accepted it on 2026-08-27.

## What was checked

- **24 judgment tests** in `compiler/src/semantic/tests/staged_permission.rs`,
  every condition in both directions. Each denial fixture violates exactly one
  numbered condition and asserts *that* condition, so a denial arriving for the
  wrong reason fails the test.
- **The loan column's closed holes, re-attacked.** This judgment admits body
  shapes [PAR-2] refuses outright — an uncounted loop, an early typed exit, a
  write of enclosing storage that is not an accumulator — so its neighbourhood
  is wider than the one batch 0081 audited and each hole had to be shown closed
  again: two `&uniq` of one cell with reads-only rows, a `pure` `&uniq` with
  zero footprint, an interposed statement after the submission, a body-bound
  borrow of enclosing storage, and the wrong-denial counterpart 0081's own
  attack found — a body-bound borrow of *iteration-own* storage, which must be
  admitted and is.
- **Facts-off identity**, pinned by a test rather than asserted in a comment.
  The compiler has no facts-off switch, so the differential is over programs:
  three staged loops whose subscript obligation is discharged three different
  ways — a constant, a dominating branch, and a retained `claim` — must produce
  identical verdicts and identical disposition tables. A judgment that read the
  fact state could tell them apart; this one may not.
- **Three ledger tests** in `compiler/src/driver.rs`: the granted table in full,
  a denial's condition/place/form, and that a loop without I/O gets a counted
  line and no staged line. The granted-table test also asserts the `PAR loop`
  line beside it, where [PAR-2] denies the same loop over the factory loan
  [PAR-3] admits — the two rules disagreeing, in one ledger, on purpose.
- **`many_files_loop.wf` publishes the bench checksum**:
  `make -C research/experiments/io-completion-bench verify` reports
  `all lines publish 17098009301725298919 00000000000071024640` with the new
  program in the list. Its helper functions are byte-identical to the other
  programs', so the pair with `many_files_narrow.wf` isolates exactly the
  hoisting.
- **No acceptance verdict and no existing ledger line moved.** The staged table
  is a separate list on `FunctionPermissions`; `judge_staged` reads no [PAR-2]
  verdict and no pair verdict, and `analyze_permission`'s existing output is
  computed exactly as before.
- Canonical `make check` at the repository root is green end to end; `cargo
  clippy --all-targets` and `cargo fmt --check` are clean.

## Spec v0.38 (activation in this branch, approval at merge)

One added rule, 137 to 138. The owner ruled on 2026-08-27 that the staged
permission is a **new rule** rather than an amendment of [PAR-2]: the two share
no condition, and rule-per-judgment legibility is worth the count. [PAR-1] and
[PAR-2] each gain one cross-reference sentence and no condition. The grammar,
the resolution catalog, the operation set, and the opaque nominal set are all
untouched — there is no new spelling, so the generated grammar tables are
byte-identical and the grammar gate confirms it.

[PAR-3] carries §4.1's text adapted to a standalone rule, plus the exhaustion
sentence distinguishing execution resources from descriptors the program's own
opens consume, plus [PAR-1]'s erroneous-execution clauses with T3's no-latch
sentence. The [SYS-8] observed/defined sentences and the [SYS-2] `begin_submit`
milestone are deliberately **not** in this version: they belong with batch
0089's milestone and with the privatization batch, and adding them here would
put rule text in front of the implementation that makes it true.

v0.37 is archived byte-exact as `spec/kernel-spec-v0.37.md`; the chain, the
generated identity, the qualification review note, and every digest anchor name
v0.38 at `7d30a62566e96b659fe7be6fb6d5775f25fb576374396bff5d1f42542b8a4e4c`.

## Approval classes for the merge

- **Specification bytes change** (v0.38 activation): the merge-time record is in
  `governance/APPROVALS.md` and becomes effective with the owner's merge
  approval of this exact revision.
- **Conformance content changes**: relative to the v0.37 activation boundary at
  `main` tip `79b2966562e1da8de541feedfd5855d0ef4a3c30`, seven case files are
  added under `tests/conformance/cases/` and `tests/conformance/manifest.jsonl`
  is modified; nothing is deleted or renamed. In the manifest, eight records are
  added — one [PAR-3] rule annotation and seven case records — and no record is
  modified or removed. No pre-existing `expect` verdict changes. Coverage moves
  from 137/137 to 138/138. The exact boundary is recorded in
  `governance/APPROVALS.md`.
- **No new root entries.**

## Named dependencies (breaking one breaks this silently)

- **[SYS-2]'s `terminal` milestone.** Condition 3 reads every retained borrow of
  a `may-suspend` call as retained to `terminal`, which is what the contract
  publishes today. Publishing `loan-released(name)` at `begin_submit` — batch
  0089's work — widens what condition 3 admits, and it is sound only after the
  two latent bugs the design names are fixed: the adapter retains the caller's
  path pointer, and `%component` is one static buffer per call site. Do not
  amend the condition before both land.
- **[EFF-2] row exactness in both directions**, which is what makes a loan with
  no declared use unobservable at value level and therefore makes the loan
  column's answer complete.
- **[SYS-10]'s call-scoped factory loan.** If reservation ever retained a loan
  past its own return, the serialized disposition on `files` would be wrong and
  the target program would have to replicate a quota it cannot replicate.
- **`buffer_new` fills only copy elements.** Condition 6 reads
  `CheckedFlatElement` directly; `buffer_vacant`'s interned `Option<T>` element
  fails closed because this judgment does not resolve a nominal's [OWN-1] class.
- **The judgment reads no entailment fact.** The moment a later batch's
  byte-range analysis lands, it must live outside `analyze_permission` and
  outside this module, because `permission.rs` and `loop_permission.rs` both
  state fact-independence as an invariant and this module now states it too.
