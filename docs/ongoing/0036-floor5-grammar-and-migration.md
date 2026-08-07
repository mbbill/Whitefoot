# 0036 — FLOOR-5 grammar path and corpus migration

This is a temporary live coordination record, not execution authority.

- **Status:** `BLOCKED` (2026-08-07: the approved delta's site sweep misses ten
  normative sites in five rules; FN-4's mandatory law-discharge body shape
  becomes unspellable, which no acceptance-set section predicts and which no
  mechanical transform can repair. Step 1 assembled through all anchored sites;
  steps 2–5 not started, because the retained-argument class may move.)
- **Authority:** owner approval 2026-08-07 (`governance/APPROVALS.md`); the
  candidate `governance/spec-evolution/spelling-relief-candidate.md`
- **Owner / workspace:** exec-0036 / `/Users/bytedance/do_not_scan/wf-0036`
  on branch `task/0036-floor5-grammar-and-migration`
- **Base revision:** b345e2c
- **Dependency:** none

## Goal

0031-style atomic activation prep for the FLOOR-5 spelling batch, on one
task branch: (1) assemble `kernel-spec-v0.23-candidate.md` from active
v0.22 plus the FLOOR-5 delta; (2) extend the grammar path — two keywords,
twenty operator spellings, the left-factored `expr`/`infix_tail`, the
if/value_if productions, deletions for targs and let annotations, FORM-2
layout; expected verifier total 69 productions (reviewer carry-forward:
production `infix_tail` maps to node kind `infix`, a name mismatch the
generator may not expect — make it a success criterion, not a surprise);
(3) repoint identity pins; (4) migrate the corpus: 1353 targ deletions,
1748 let annotations, 257 Bool matches to if/else with mandatory else-if
flattening, ~384 infix respells — scripted in scratch, every file passing
the branch compiler's parse + FORM-2 canonical audit, plus a REVIEW PACKET
(diffstat, ten representative before/after excerpts, verdict-meaning
statement); (5) evidence: verifier green on branch, `make -C compiler
check` and repo `make check` exit 0 (direct exit codes), adapter
comparison; (6) STOP before merge with the candidate SHA-256.

Discoveries outside the candidate stop the task with evidence.

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
