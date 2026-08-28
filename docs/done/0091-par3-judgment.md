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
its argument evaluation and submission, P for the statements up to and including
c, and E for the rest. c is a **program point** inside the statement that
performs it, not the whole of that statement: the argument evaluation and
submission end P, and the *outcome* of that submission, which only E joins, is
after c. The schedule the rule admits is: P(0), P(1), … in index order, never
two at once; E(i)'s stages executed after P(i+1..i+K) may already have run;
E(i)'s accesses to places rooted outside the loop that the body writes — its
reads as well as its writes — taken in the order of i.

Two sentences of that schedule are load-bearing and neither is derived:

- **Prologues never overlap one another.** This is what admits
  `reserve_file(&uniq files)` with no replication and no loan exemption. At
  every program point exactly one unique loan of the factory is live, so
  [OWN-5] is not relaxed — [SYS-10] already says the factory loan ends when the
  inline operation returns, and index-ordered prologues make those windows
  disjoint and in source order.
- **E's accesses to storage rooted outside the body that the body writes are
  taken in iteration order — its reads as well as its writes.** The write half
  is what admits `set sum = sum +wrap digest;` as an ordinary source-order write
  with no associativity, no identity element, and no combination tree; the read
  half is what makes a cursor the remainder reads and then advances safe to
  grant, and it was added to the rule after the adversarial review below. It is strictly more general than [PAR-2]'s accumulator
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
| **read-only** | no footprint of B writes it *or any place overlapping it*, and every loan on it is shared | `cwd` in every bench program: `open_file`'s retained `&cwd` is shared, and two overlapping shared loans deny nothing |
| **serialized** | every footprint element and every loan touching it belongs to one segment alone, and no loan on it is retained past the cut | `files` in `many_files_loop.wf` (prologue, through `reserve_file`'s inline loan); `sum` and `bytes` (remainder, whose accesses to storage rooted outside the loop are taken in index order) |
| **replicated** | copy element type, no continuation read, and every byte read was written earlier in the same iteration | `name` and `data` in `many_files_loop.wf`, which the body itself constructs |
| **denied** | everything else | `data` in `many_files_narrow.wf`; the enumeration cursor; storage reached on both sides of the cut |

Conditions 3 and 4 are stated over places, so their failures are the fourth
disposition rather than a fifth column: a place a `may-suspend` call retains a
borrow on and the body writes, or one a call of the remainder holds an exclusive
loan on, has no safe disposition.

Every flag these conditions read is accumulated over the place's whole [OWN-7]
overlap class before any disposition is taken. A row is still keyed by the exact
resolved path, so the ledger names the place the writer wrote; only the flags
are unioned. Reading them per path instead is the first widening the adversarial
review of 2026-08-27 found, and it is recorded below.

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

Two line kinds, both anchored at the **loop head** when the checked tree
carries one — a `for_stmt` does, a `loop_stmt` does not and falls back to the
cut:

```text
PAR stage       staged.wf:3  for   permitted   staged at open_file<'f, 'n>(…); 4 places classified
PAR place       staged.wf:3  serialized-P  &uniq 'f files  every footprint element and loan touching it belongs to the prologue, and prologues run in index order without overlapping
PAR place       staged.wf:3  read-only     &'f cwd  no footprint of the body writes it or any place overlapping it, and every loan on it is shared
PAR place       staged.wf:3  serialized-E  set total = total +wrap 1_u64;  every footprint element and loan touching it belongs to the remainder, whose accesses to storage rooted outside the loop are taken in index order
PAR place       staged.wf:3  replicated    let name = buffer_new(16_u64, 97_u8);  iteration-own storage with copy elements, which an implementation may give each in-flight iteration its own of
```

The head is what tells two nested loops apart. When the inner loop holds the
body's only submission, that call is the outer loop's first submission too and
both judgments cite it; anchoring the line on the cut printed two verdicts at
one source position and a reader could not tell which loop either belonged to.

A denial names the numbered condition, the place, and **one admitted writer
form**, and the form comes from the judgment itself so it cannot drift from the
condition that produced it:

```text
PAR stage  hoisted.wf:5  for   denied  condition 3: a may-suspend call retains a borrow
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

`let handle = propagate open_file(…);` is the same residual written more
compactly, and it is refused for the same reason: the `Err` edge is selected by
the submission's own outcome, which only the remainder joins. The first
implementation granted it, which is the second widening below.

Closing it needs FIRST-PRINCIPLES §8's closed-state proof extended to a target
call whose *result* is a fresh resource; per-iteration allocation supplies the
destination half of that proof and not the resource half. This is recorded as a
named residual, not solved, and the owner accepted it on 2026-08-27.

## The adversarial review of 2026-08-27, and what it changed

An adversarial reviewer built and ran 34 programs against the first
implementation and **refuted** it. Every one of those programs now lives in
`compiler/src/semantic/tests/staged_permission_corpus.rs`, verbatim, with the
verdict it must carry, including the ones the first implementation already
judged correctly — a corpus that kept only the failures would not notice a
repair that traded one widening for an over-denial somewhere else.

**Widening 1: places were keyed by exact path, not by the [OWN-7] relation.**
`entry.place == *place` made `work` and `work.seen` two independent rows and
handed each a safe disposition on its own: `work.seen` read-only because
nothing writes *that path*, `work` serialized-E because nothing else touches
*that path*, while the body carried a recurrence through the one storage they
share. `A19-field-recurrence.wf` was granted and the byte-identical recurrence
in a bare `u64` was correctly denied — the same program, one type change,
opposite verdicts, in the unsound direction. `A26-struct-name-swap.wf` was the
severe form: `open_file(name: &'n held.name, …)` retains a borrow into a buffer
the remainder then drops by replacing `held`, while a later iteration's open is
still outstanding on it. `A34-mirror-prologue-write.wf` showed serialized-P was
unsound by the same mechanism, not only read-only, and `A28-nested-field.wf`
showed the hole was not depth-limited.

*Repair.* `Class` (`compiler/src/semantic/staged_permission.rs`) unions
`written`, `in_prologue`, `in_remainder`, `exclusive_loan`, `retained_borrow`,
`remainder_exclusive_loan` and `replicable_shape` over every row whose place
`ResolvedPlace::overlaps` the subject's, and every condition reads the class
rather than the row. The union only ever adds flags, so it can turn a grant into
a denial and never the other way round. A denial the class decided names both
halves of the pair — `, which overlaps <the other statement>` — and carries its
own writer form, because a writer looking at two statements that mention
different paths has to be told they are one storage before any other advice
reads as advice. The half a condition-3 denial names is the *write*, tracked as
`Class::written_at` rather than as the first place that widened the class: with
a borrow of one field and a write of a sibling field reached through a loan on
the whole record, the first widener is the borrow itself, and printing it as its
own counterpart tells the reader nothing.

Reading condition 3 over the class is one step coarser than the rule's words,
which ask whether a footprint writes *the borrowed place*. It costs no verdict,
only the condition number: whenever the class fires condition 3 where the rule
would not, the subject place is one a footprint writes and one a loan retained
past c touches, so condition 5's first alternative fails on the write and its
second on the retained loan, and the loop is denied at that place regardless.
That argument is written into the module doc beside the code that relies on it.

**Widening 2: a `propagate` whose right-hand side is the cut was counted as a
prologue exit.** The cut statement was assigned `Segment::Prologue` by
`id == cut` and exits were reported only from the remainder, so a statement that
both submits and leaves was never reported. `A20-propagate-cut.wf` was granted;
`A20b-match-twin.wf`, the same exit written as an `Err` arm returning, was
denied. That decided a language capability by source shape and silently defeated
the W1 residual the owner accepted.

*Repair.* c is a program point inside the statement that performs it, not the
whole statement. The statement's argument evaluation and submission end P; the
*outcome* of that submission, which only E joins, is after c. So the cut
statement's footprint stays the prologue's and its **leaving edge** is the
remainder's, and condition 2 refuses it. This needed one clarifying sentence in
the rule text, recorded below.

**Rule-text gap: the remainder's ordering covered writes only.**
`A09-remainder-cursor.wf` reads `cursor` and then advances it, both in E, and
feeds the read to `read_at`'s `file_offset`. The rule ordered E's writes and
explicitly permits E(i) to overlap E(j), so nothing ordered E(i)'s *read*
against E(j)'s write: the host offsets read are 0, 0, 64, 128 sequentially and
can be 0, 0, 0, 0 pipelined, while the compensating clause about final binding
values is still satisfied. The serialized-E disposition was already assuming the
stronger ordering — the module doc's own words are *either segment therefore
serializes the place*. The rule now states it. Serializing the reads is the
choice that keeps the cursor program granted; denying it would have cost a shape
a real program writes for a guarantee the schedule can simply give.

**Nested loops sharing a cut were indistinguishable.** `StagedPermission` now
carries the loop `head` and the ledger anchors on it, with the cut as the
fallback a `loop_stmt` needs. `nested_loops_sharing_one_cut_print_at_their_own_heads`
in `compiler/src/driver.rs` pins it.

**`StagedDenial::Unresolved` had no test in either direction.** It now has
both: the `slice_of` length read denies as `Unresolved` rather than as
`BodyForm`, and the identical length read taken from the buffer itself is
granted. The two programs differ only in whether a slice stands between the read
and the storage, which is what makes the denial a resolution limit of this
judgment rather than a hazard of the program.

**The two over-denials are kept, and their advice was made usable.** Both are
fail-closed and sanctioned by condition 7, and neither is relaxed here: an
expression statement's reach projects onto no actual, and a slice reads through
an origin this judgment holds no place for. What was wrong is that both were
offered one sentence listing admitted statement forms, which tells the writer of
either one nothing. `StagedDenial::BodyForm` now carries the admitted form
beside the refused one, so an expression statement is answered with *bind the
call's result with `let`*, a discarded owned result with *bind the value with
`let` and let the binding's own release carry it*, and a body-bound borrow with
*write the borrow as an argument of the call that uses it*; `Unresolved` is
answered with *name the storage the call reaches directly rather than through a
binding whose extent this judgment does not resolve*.

**A `break_stmt` denial now names which loop the break leaves.** A break carries
no node path in the checked tree, so no place can be cited; the loop it names is
the one identity it has, and the edge label carries it.

**Evidence gap.** The first implementation's tests contained no `struct`, no
field-path place, and no `propagate` — the two unsound directions were exactly
the two constructs the suite never built. Both now have granted/denied pairs of
their own, beside the whole corpus.

## What was checked

- **35 judgment tests** in `compiler/src/semantic/tests/staged_permission.rs`,
  every condition in both directions, every variant of condition 7 in both
  directions, and each of the three statement forms `BodyForm` refuses with the
  advice that form is answered with. Each denial fixture violates exactly one
  numbered condition and asserts *that* condition, so a denial arriving for the
  wrong reason fails the test.
- **The adversarial corpus**, `compiler/src/semantic/tests/staged_permission_corpus.rs`:
  all 34 programs of the review of 2026-08-27, kept verbatim, each with the
  verdict it must carry. Programs the checker rejects for a reason that is not
  this judgment's are recorded as `Rejected` rather than dropped, so a later
  change that makes one of them check has to come back and give it a verdict.
  The drifts are collected rather than asserted one at a time, so a change that
  moves several verdicts reports all of them.
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
- **Four ledger tests** in `compiler/src/driver.rs`: the granted table in full,
  a denial's condition/place/form, two nested loops sharing one cut printing at
  their own heads, and that a loop without I/O gets a counted line and no staged
  line. The granted-table test also asserts the `PAR loop`
  line beside it, where [PAR-2] denies the same loop over the factory loan
  [PAR-3] admits — the two rules disagreeing, in one ledger, on purpose.
- **`many_files_loop.wf` publishes the bench checksum**:
  `make -C research/experiments/io-completion-bench verify` reports
  `all lines publish 17098009301725298919 00000000000071024640` with the new
  program in the list. Its helper functions are byte-identical to the other
  programs', so the pair with `many_files_narrow.wf` isolates exactly the
  hoisting.
- **No acceptance verdict and no [PAR-1] or [PAR-2] ledger line moved.** The
  staged table is a separate list on `FunctionPermissions`; `judge_staged` reads
  no [PAR-2] verdict and no pair verdict, and `analyze_permission`'s existing
  output is computed exactly as before. The staged table's only consumer is
  `render_ledger`, which `check.rs` calls as pure presentation, so no verdict of
  this judgment can move an emitted byte — the repairs below change `PAR stage`
  and `PAR place` lines and nothing else.
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

Two sentences were added to [PAR-3] after the adversarial review, and nothing
else in the specification changed:

- Condition 2 gains *An edge the statement performing c takes on the outcome of
  that submission, which is the edge a `let_stmt` selecting `propagate_let_rhs`
  at c takes, is an edge of E and not of P.* Without it the rule's own
  definition of P as the statements *up to and including c* put a `propagate` at
  the cut in the prologue, which contradicts the rule's later obligation that an
  iteration leaving through an edge of P produces no observable the source-order
  execution does not.
- The schedule gains *Every read E performs of a place rooted outside B that a
  footprint of B writes likewise occurs in the order of the iterations that
  perform it.* The serialized-E disposition was already granted on that ground;
  the sentence is what makes it owed rather than assumed.

Both narrow the permitted overlap and neither moves an acceptance verdict, so no
conformance case changes. The [PAR-3] paragraph of the META-5 delta declaration,
the merge-time record in `governance/APPROVALS.md`, and the v0.38 amendment
section of `spec/derivation/derivation-ledger.md` are updated to describe the
same content.

[PAR-3] carries §4.1's text adapted to a standalone rule, plus the exhaustion
sentence distinguishing execution resources from descriptors the program's own
opens consume, plus [PAR-1]'s erroneous-execution clauses with T3's no-latch
sentence. The [SYS-8] observed/defined sentences and the [SYS-2] `begin_submit`
milestone are deliberately **not** in this version: they belong with batch
0089's milestone and with the privatization batch, and adding them here would
put rule text in front of the implementation that makes it true.

v0.37 is archived byte-exact as `spec/kernel-spec-v0.37.md`; the chain, the
generated identity, the qualification review note, and every digest anchor name
v0.38 at `3dd5878bbfe77a938fb7a9af53db97d0ba35a8e86234c3b2814b94780228ce50`.

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
