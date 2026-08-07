# 0036 — FLOOR-5 grammar path and corpus migration

This is a temporary live coordination record, not execution authority.

- **Status:** `WAITING` (2026-08-07: the round-1 blocker is ruled; waiting on the
  drafter's fixed delta before re-assembling the candidate. Read-only
  preparation meanwhile.)
- **Authority:** owner approval 2026-08-07 (`governance/APPROVALS.md`); the
  candidate `governance/spec-evolution/spelling-relief-candidate.md`; the lead's
  2026-08-07 rulings on this task's round-1 blocker report, which re-key FN-4's
  discharge premise, expand this task to full atomic activation, and sequence
  re-assembly after the fixed delta
- **Owner / workspace:** exec-0036 / `/Users/bytedance/do_not_scan/wf-0036`
  on branch `task/0036-floor5-grammar-and-migration`
- **Base revision:** b345e2c
- **Dependency:** none

## Goal

0031-style **full atomic activation** for the FLOOR-5 spelling batch on one
task branch. The original card scoped this task to the grammar path only while
still demanding a green compiler gate and adapter parity; the lead's 2026-08-07
ruling accepts that the card was under-scoped and expands the task to the whole
activation. Scope:

1. Re-assemble `kernel-spec-v0.23-candidate.md` from active v0.22 plus the
   **fixed** FLOOR-5 delta (FN-4's discharge premise re-keyed to the
   operand-derived selected type, the ten uncovered prose sites, GRAM-1's
   shape-kind enumeration, and the drafter's re-sweep for the same miss class).
2. Extend the grammar path — the `if` keyword, twenty operator spellings, the
   left-factored `expr`/`infix_tail`, `if_stmt`/`value_if`, the targ and
   let-annotation deletions, FORM-2 layout. Expected verifier total 69
   productions. Reviewer carry-forward as an explicit success criterion:
   production `infix_tail` maps to node kind `infix`, and every other
   production/node pair in this grammar shares a name, so this is where a
   generator deriving node kinds from production names breaks.
3. Extend the **semantic path**: TYPE-5 statement-local derivation replacing
   written annotations, OP-2 operand-derived row selection, GIVE-1 derived
   delivery with the empty-delivery-set rejection, GRAM-6 type-driven
   conditional forms, and `if_stmt`/`value_if` through resolution, checking,
   ENT-3 S1 branch facts, ENT-5 joins, lowering, and backend. FN-4's discharge
   shape in `calls.rs` and `catalog.rs` follows the re-keyed premise.
4. Repoint identity pins (0029/0030 style).
5. Migrate the corpus by scripted transform in scratch: deleted-class type
   arguments, body-let annotations, Bool matches to if/else with mandatory
   else-if flattening, infix respells. Every migrated file passes the branch
   compiler's parse and FORM-2 canonical audit; conformance cases migrate in
   the same change and no verdict changes meaning.
6. Evidence: verifier green on the branch with the expected production total;
   `make -C compiler check` and repo `make check` exit 0, exit codes read
   directly and never through a pipe; conformance adapter comparison against
   main's 386/1/14 lane. Plus the REVIEW PACKET — per-file diffstat, ten
   representative before/after excerpts covering every transform class, and the
   verdict-meaning statement.
7. STOP before merge, reporting the candidate SHA-256.

Discoveries outside the candidate stop the task with evidence. If the expanded
scope proves too large to finish reliably, hand back at a clean boundary as
0031's first executor did.

## Progress

Step 1 only. `governance/spec-evolution/kernel-spec-v0.23-candidate.md` is
assembled from `spec/kernel-spec-v0.22.md` by a scratch script (deleted after
use) applying 57 verbatim anchors, each asserted to occur exactly once. Rule
count 128 unchanged, sections 20 unchanged, no let annotation survives in a
normative program, and the frontend contract changed only in the
`[FORM-1]`..`## 4. Types` section (22819B -> 26914B); the `[CONST-1]` and
`[EFF-1]` sections are byte-identical, as the delta predicts. Candidate
SHA-256 `9ca13585dca6668bec9c1b4c219cbb5fc94559bcc29f67d04d52bbb61af5eef4`.
These bytes are NOT ready for step-4 approval; see the blocker.

Steps 2–5 not started. They are deliberately not started rather than partly
done: the blocker's likely repair moves TYPE-5's retained-argument class, which
would change the terminal inventory, the grammar tables, the migration counts,
and the candidate hash.

## Blocker

The approved delta enumerates 24 rules at 46 anchored sites and its §4 claims
the acceptance-set change is "one canonical respelling plus two deliberate
narrowings". Both statements are incomplete: the sweep covers the 24 rules it
modifies but not the rules whose normative prose *uses* the respelled
operations. Ten normative sites in five unlisted rules still write a deleted
type argument or a respelled spelling, measured on the assembled candidate:

| rule | line | dead spellings |
|---|---|---|
| STOR-2 | 299 | `box_new<T>` |
| STOR-5 | 313 | `box_new<T>` |
| OP-2 | 384 | `iadd.wrap<T>` `isub.wrap<T>` `imul.wrap<T>` |
| OP-2 | 386 | `iadd.trap<T>` `isub.trap<T>` `imul.trap<T>` |
| OP-2 | 388 | `ieq<T>` `ine<T>` `ilt<T>` `ile<T>` `igt<T>` `ige<T>` |
| OP-2 | 392 | `ineg.wrap<T>` `ineg.trap<T>` `ineg.checked<T>` |
| OP-9 | 410 | `buffer_new<T>` |
| FN-4 | 446 | `iadd.sat<D>` |
| FN-4 | 452 | `iadd.sat<T>` |
| FN-4 | 453 | `iadd.sat<T>` |

Plus FORM-3 line 68, whose OPNAME example is `iadd.checked` — a spelling the
batch deletes.

Nine of the eleven are mechanically forced respellings with no design choice,
though OP-2's four make the rule self-contradictory as assembled: the
candidate's own rewritten judgment paragraph opens "Each operation in the
preceding paragraphs carries no written type argument", and those preceding
paragraphs write the type argument.

**FN-4 line 446 is the blocker and is not mechanical.** It reads: "the bound
function's body must contain exactly one statement, `return iadd.sat<D>(p0,
p1);` … and the explicit type argument is D." Under the batch `iadd.sat`
respells to `+sat` and its type argument is deleted, so the one mandated body
shape has no legal spelling and every source conformance-law discharge becomes
impossible. Reproduction, `tests/conformance/cases/fn4-pos-law-discharged.wf`:

```
fn satadd(x: own u64, y: own u64) -> own u64 pure {
  return iadd.sat<u64>(x, y);
}
```

The old bytes reject under OP-1/TYPE-5; the migrated bytes `return x +sat y;`
reject under FN-4's literal mandate. Live impact: 6 corpus sites, 4 conformance
cases (`fn4-pos-law-discharged`, `fn4-pos-law-in-contract`,
`fn4-neg-law-undischarged`, `fn4-neg-law-refuted-signedness`), and the
compiler's own discharge shape.

The repair is a decision, not a transform, because "the explicit type argument
is D" is a *premise of the discharge relation*: either FN-4 re-derives D from
operand agreement under the new OP-2, or `iadd.sat` joins TYPE-5's
retained-argument class — which the review certified "total against the
complete operation table", so moving it reopens that finding. Owner or lead
ruling needed.

## Deviation applied and reported

One 47th anchored site was applied beyond the approved 46, in GRAM-1. Adding an
operator form to GRAM-1's maximal-form list leaves the shape-kind enumeration
("Raw formation gives every token exactly one context-free shape kind: …")
false for operator tokens, and that enumeration is the interface terminal
membership and the compiler's `TokenKind` are written against
(`compiler/src/lexer/token.rs:63`). The candidate's header accounting was
updated to "forty-seven" and GRAM-1 relabelled "four sites" in the same change.
This is a byte the owner did not review and it returns to review with the rest.
