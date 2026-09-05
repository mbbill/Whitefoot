# Evidence — sweep D1, and the D4 join-image rule

Design evidence for the container and resource work. D1 is the sweep's unsound
accept at the call boundary; it is recorded here because the container design
must not repeat its mistake. D4 is recorded in one paragraph because the
container design's loops will meet it.

Sources:

- The scenario-sweep defect ledger of 2026-09-03 (`LEDGER.md`, a session artifact that is not in the repository; the `LEDGER.md:` line references below point into it, and everything this file relies on from it is quoted here), section D1 (`L28-111`) and section D4 (`L232-282`).
- `tests/conformance/cases/ent5-neg-callee-uniq-buffer-replace-kills-length.wf`
  and its manifest entry, `tests/conformance/manifest.jsonl:165`.
- Specification line numbers below are `spec/kernel-spec.md` in this worktree
  (ACTIVE v0.40). The ledger's own numbers are against the same version in the
  sweep worktree and differ by a few lines in places.

---

## D1 — a callee's write through a `&uniq buffer<T>` actual never kills the buffer's length fact

### The minimal program

The conformance case is the ledger's minimal repro. It is accepted today, is
claim-free and `unsafe`-free, and performs an out-of-bounds heap read:

```wf
fn shrink['a](handle: &uniq 'a buffer<u8>) -> discarded: own buffer<u8>
    reads(handle), writes(handle), allocates(heap) {
  let smaller = buffer_new(2_u64, 0_u8);
  let old = replace deref(handle) = move smaller;
  return move old;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let line = buffer_new(10_u64, 0_u8);
  region 'r {
    let dropped = shrink<'r>(handle: &uniq 'r line);
  }
  let tail = line[9_u64];
  return exit_status(code: 0_u8);
}
```

`buffer_new(10_u64, ...)` establishes `len(line) = 10`. The callee replaces the
whole referent with a two-byte allocation. The caller keeps the stale length and
uses it to discharge offset 9 of what is now a two-byte object.

### The controls

Each was re-run against the sweep binary while the ledger was written
(`LEDGER.md:53-69`):

| control | shape | verdict | what it isolates |
|---|---|---|---|
| `ctlA.wf` | same program with `line[10_u64]` | reject, `UndischargedBoundsObligation { residual: "10_u64 < len(line)" }` | the checker still holds exactly `len(line) = 10` after the call; the stale fact is what discharges offset 9 |
| `ctlW.wf` | `set line[9_u64] = 7_u8;` after the call | accept | the defect is not read-only: it admits a store eight bytes past the end |
| `ledger-probe/direct.wf` | the identical `replace` written directly in `main` | reject | only the call boundary leaks; the direct `replace` path is correct |
| `ledger-probe/callnoregion.wf` | subscript inside the region | accept | region exit is not part of the trigger |
| `ledger-probe/intfact.wf` | the same call shape with `&uniq 'r idx` on a `u64` local | reject (`idx < len(buf)`) | the [ENT-5] clause (b) call kill does fire for ordinary place facts; length facts specifically survive |

The last two rows are the load-bearing pair: the kill machinery works, and the
classification of one flag is what fails.

### The mechanism

The `element` flag on a projected call write is derived from the **actual's
syntactic shape**, not from what the callee does (`LEDGER.md:83-95`):

- `compiler/src/semantic/places.rs:349-355`, `argument_referent`, returns
  `Some((self.resolve(&place), true, false))` for
  `CheckedExpression::BorrowBuffer` — every callee write through a
  `&uniq buffer<T>` actual is classified as an element write.
- `FunctionEntailment::event_kills_term`
  (`compiler/src/semantic/entailment/flow.rs:2927`; the `TermKind::Length` arm at `2951-2956`) then
  returns `false` for `KillEvent::Write { element: true, .. }`, under the comment
  "An element write never kills a length fact".
- The sibling arm for `BorrowAddressed` / `BorrowBox` / `BorrowSystemResource`
  passes `element = false`, which is why the `&uniq u64` control rejects.
- The holder-binding arm (`Binding` with `Buffer`/`Slice` type,
  `places.rs:394-398`) carries the same classification and is likely the same
  bug for reborrowed holders.

### The specification rules

- **[ENT-5] support**, `spec/kernel-spec.md:2857`: for a length term `len(P)`,
  the support is the root binding of `P` but not `P`'s element storage — "an
  element write never kills a length fact".
- **[ENT-5] death clause (a)**, `spec/kernel-spec.md:2887`: because a length
  term's support is its viewed place's non-element root path, "a whole-place
  replace of a buffer or of any prefix of it kills that buffer's length facts,
  while an element-position replace, like an element write, kills none". The
  callee's `replace deref(handle)` is the whole-place case.
- **[ENT-5] death clause (b)**, same line: a fact dies at a call one of whose
  [EFF-2] boundary-projected `writes` occurrences projects onto a caller place
  overlapping [OWN-7] the resolved place of any support member — "a callee
  writing only through one `&uniq` actual kills exactly the facts whose support
  overlaps that actual's resolved place".
- **[EFF-2] projection**, `spec/kernel-spec.md:1405`: each callee effect path
  selects its root formal's actual argument and appends its static field suffix
  to that actual's resolved place, and holder resolution reaches the borrowed
  referent. `writes(handle)` therefore projects onto the place `line`, which is
  exactly `len(line)`'s support member.
- **[ENT-3.S-source]** established `len(line) = 10` at the allocation.
- The correct verdict is a rejection at **[OP-4]** with residual
  `9_u64 < len(line)`.

The specification and the compiler disagree; the specification is right. This is
a compiler defect, classified unsound accept, severity blocking
(`LEDGER.md:30-33`).

### The recorded conformance case

`tests/conformance/cases/ent5-neg-callee-uniq-buffer-replace-kills-length.wf`
carries the program above with a `doc` string stating the required verdict.
`tests/conformance/manifest.jsonl:165` records
`"rules": ["ENT-5", "EFF-2", "OP-4"]`, `"expect": {"kind": "reject", "rule":
"OP-4"}`, `"status": "xfail"`, with a reason that names the mechanism: the
compiler derives the element flag from the actual's syntactic shape, so no
length fact ever dies at such a call, the stale length discharges the subscript,
and the program is wrongly accepted. The manifest states the case "tracks that
gap until the classification fix or the container and ownership redesign lands,
and turns into an XPASS the moment either does" — so the container work owns
this case's disposition.

### The lesson: a surviving fact must come from signature-visible information

`writes(handle)` in the effect row does not certify element-only writing. A
callee is free to `replace deref(handle) = ...`, which is the [ENT-5] clause (a)
whole-place replace, and nothing in the signature distinguishes the two
(`LEDGER.md:96-111`). The defect is exactly the caller inferring a
fact-preserving property — "this write was an element write" — from the
*syntactic shape of its own argument expression*, which is information about the
call site, not about the callee. The ledger's minimal sound repair is to return
`element = false` for the `BorrowBuffer` and buffer/slice-holder arms, so every
projected callee write through a `&uniq buffer<T>` actual kills the referent's
length facts; the precision-preserving repair is to record a per-parameter
"replaces the whole referent" bit on the checked callee summary and set the flag
from that bit — that is, to make the distinction signature-visible before relying
on it.

The container design inherits this as a constraint, because it multiplies the
number of facts a caller wants to keep across a call. Every fact a caller expects
to survive a call — a container's length, its capacity, its initialized prefix,
the remaining spare of an append view, the disjointness of a range handed to a
parallel lane — must be justified by something the callee's declared signature
makes visible: its parameter capability, its effect row, or an explicit
postcondition. A capability that promises "this operation cannot change `len`" is
a promise the signature has to carry; if the boundary between a length-preserving
and a length-changing operation is not in the signature, the caller may not act on
it, and the analogous mistake produces the analogous out-of-bounds access. The
same reasoning is why the discussion's append capability may never grow and why
growth is a separately named owner-level operation with its own effect: the
distinction is in the signature, where a caller can see it.

`LEDGER.md:311-317` records the same principle from the other direction, in a
finder's error: a function is judged on its own signature — "No caller fact is
copied into a callee" — and a program rescued in `main` by a small literal buffer
is still an unsound accept when the callee's own signature admits the overflow.

---

## D4 — the join-image rule, as a writer-facing constraint on the container's loops

D4 is not a defect: the compiler and the specification agree and the rejections
are mandatory (`LEDGER.md:234-238`). The rule is that every arm of a loop-body
join must leave a tracked binding with the same affine image, or with images
differing only by a constant; otherwise the guard fact dies at the join and the
header invariant cannot be re-established. [ENT-6]'s value-image join keeps an
identical image, and images sharing one nonconstant coefficient vector and
differing only in their constant join to that vector plus a fresh delta atom over
the incoming constant range; anything else gives the binding a fresh full-type
atom. [ENT-5]'s all-predecessor join then keeps only the bounds held on every
input, so the guard fact — the very correlation the writer's argument used — is
discarded, and [INV-1] proves the next header target over the joined images with
[ENT-1]'s four AUTO families, which cannot recover it (`LEDGER.md:251-258`). The
verified shapes are: identical update on every arm accepts; arms adding different
constants accept; one arm advancing the binding while another leaves it rejects;
arms assigning different non-constant values reject; a per-arm or tail
`invariant_stmt` restating the target rejects, because the published conclusions
are not canonically identical; re-exposing the fact with a dominating guard after
the join accepts; and lifting the choice into the addend with `value_if` plus a
bound on the addend accepts (`LEDGER.md:261-271`). This matters for containers
because the ordinary append loop is exactly this shape whenever an element is
appended only under a condition: `len` is then advanced on one arm and not the
other, and the capacity invariant at the loop header will not survive unless the
writer uses one of the accepted shapes. The doctrine is now `docs/patterns.md`
P19, and its two verdicts are pinned by
`tests/conformance/cases/ent6-neg-join-one-arm-advances-accumulator.wf` and
`tests/conformance/cases/ent6-pos-join-value-if-lifted-addend.wf`.
