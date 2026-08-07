# 0036 — FLOOR-5 grammar path and corpus migration

This is a temporary live coordination record, not execution authority.

- **Status:** `WAITING` — handed back 2026-08-07 at a clean boundary, before any
  compiler file was touched. Nothing is half-built: the branch holds only the
  assembled candidate and this record, and `make check` exits 0. A successor
  takes over from the hand-back brief below. This is earlier than the
  pre-authorized "grammar path + pins green" boundary; the reason is in the
  brief.
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

## Read-only preparation (round 2, at 46c1a0c)

Semantic-path site map, measured not estimated. The load-bearing finding is
that `if`/`value_if` is far cheaper than the card implies: `CheckedEnumType`
already has a `Bool` variant and `check/control/matches.rs:193` already builds
a Bool `MatchDescriptor`, so both forms check into the *existing* checked
Bool-match statements. Entailment (`entailment/flow.rs:208,219,958,978,1155,
1163`), lowering (`lowering/builder.rs:506,519`), cleanup, drops, and backend
all consume `CheckedStatement::Match`/`ValueMatchLet` and need no change — ENT-3
S1 and the ENT-5 join come out isomorphic for free, which is what the candidate
asserts. GRAM-1's 1:1 production-to-node law governs the source tree; the
checked IR is compiler-internal, so this is not desugaring in the spec's sense.

- `if`/`value_if` new work: syntax productions and nodes, the FORM-2 printer,
  and GRAM-6's three new rejections in `check/control.rs` and
  `check/control/matches.rs` (Bool-scrutinee `match`, empty `else`, else-if
  flattening).
- Let-annotation deletion: `check/control.rs:443 check_let` reads the `Mode`
  and `Type` children and threads the declared type into
  `check_match(.., Some(expected))`. The new GIVE-1 inverts that direction —
  the delivery set produces the type by agreement, so the descriptor returns
  the derived mode and type from the first delivering `give`, later `give`s are
  checked for exact agreement, and the empty delivery set rejects at the
  `let_stmt` node. The OWN-5 `SliceValueMatch` guard and the propagate path
  re-key onto the derived type.
- Type-argument deletion: 31 `Production::Targs`/`Targ` read sites across
  semantic and resolution; the table-op ones concentrate in
  `check/expressions/calls.rs` (4). Retained-class readers stay. FN-4's
  discharge shape sits at `check/expressions/calls.rs:150` and
  `resolution/catalog.rs:108` and follows the re-keyed premise.
- FORM-2 printer: `finalize/canonical/format.rs` is table-driven
  (`is_line_bearing:18`, `is_block_bearing:58`). Adding the two productions is
  one line each; the new mechanism is the join line (`} else {`,
  `} else if … {`), since no production renders a close-and-open brace line
  today. The `fn … requires` `} {` line is the precedent to follow.
- Grammar tables are the real risk: `syntax/grammar/generated.rs` is 4131 lines
  of committed LL(2) data (65 productions, 75 decisions, 530 nodes, 364 select
  atoms, 2003 select rows) with no in-repo generator; 0030 and 0031 both used
  one-shot scratch generators. Going to 69 productions and +21 fixed terminals
  (76 -> 97 predicates, still inside `TerminalSet`'s u128) moves FOLLOW sets
  broadly, because operator tokens newly follow every atom — `psuffix*`,
  `place`, `atom_list`, and `expr` rows all shift. Plan: build the generator so
  it reproduces v0.22's existing tables byte-for-byte first, then extend. The
  reviewer's `infix_tail`/`infix` name-mismatch carry-forward is a property of
  that generator, so it is a first-class test of it.

## Hand-back brief

Read this section and the two above it; the rest is history.

**Why the hand-back is here rather than at "grammar path + pins green".** The
generator that ruling 2 makes a hard gate is an iterative build — reproducing
530 nodes, 364 select atoms, and 2003 select rows byte-for-byte means many
diff-and-adjust cycles against the committed table, each one cheap on its own
and expensive in aggregate. Starting it with the budget left would have
produced a half-built generator that a successor must first understand and then
probably discard, which is worse than a clean hand-off. The fixed delta had
also not landed, so the extension target was still unknown. Everything below is
work a successor would otherwise have to redo.

**1. The single most valuable finding: `if`/`value_if` is cheap.**
`CheckedEnumType` already has a `Bool` variant and
`check/control/matches.rs:193` already builds a Bool `MatchDescriptor`, so both
new source forms check into the *existing* checked Bool-match statements.
Entailment, lowering, cleanup, drops, and backend need no change, and ENT-3 S1
plus the ENT-5 join come out isomorphic for free. The lead approved this on
2026-08-07: GRAM-1's 1:1 production-to-node law governs the source tree, while
DIAG-2 makes the checked program private compiler state, so two source
constructs sharing one checked form is an implementation choice, not
source-visible desugaring.

**2. Its two approval conditions.** (a) Every diagnostic and DIAG-3 path must
resolve to the `if` source node — **verified, not assumed**: `issue_node`
(`check/support.rs:121`) takes a source `NodeId` and resolves it through
`self.tree.path(node)`; every `TrapSite` is built in the checker the same way
(`control.rs:268`); and `grep -c NodePath` over `compiler/src/lowering` and
`compiler/src/backend` returns **0**, so nothing downstream can override the
location. Pass the `if_stmt` node for the empty-else and flattening
rejections, the condition `expr` node for a condition failure, and the
scrutinee `expr` node for a Bool-scrutinee `match`. (b) Pin it with negative
conformance cases asserting both the cited rule and that the citation lands on
the `if` construct: `gram6-neg-bool-scrutinee-match`, `gram6-neg-empty-else`,
`gram6-neg-unflattened-else-if` (citation at the *nested* `if_stmt`, per the
candidate), plus `give1-neg-empty-delivery-set` at the `let_stmt` node.

**3. Grammar source map — includes a non-obvious trap.** The 65 productions do
not all live in fenced EBNF blocks. 61 do: GRAM-2 has 26, GRAM-3 has 5, GRAM-4
has 18, GRAM-5 has 12. The remaining **four are written inline in prose** with
backticks — `const` in [CONST-1], `cvalue` in [CONST-2], and `effects` and
`effect` in [EFF-1]. That is why the verifier's frontend contract hashes the
prose ranges `[CONST-1]`..`## 5. Ownership` and `[EFF-1]`..`[EFF-2]` rather
than code blocks, and it maps exactly onto the `RuleOwner` enum. A generator
that scrapes only fenced blocks silently produces 61 productions.

**4. Generator plan (ruling 2).** Build in scratch, delete after use, per 0030
and 0031 precedent. Milestones: parse the EBNF above into node trees; reproduce
`PRODUCTIONS`, `PRODUCTION_ROOTS`, `PRODUCTION_OWNERS`; then `GRAMMAR_NODES`
(530) and the children/terminal arenas; then `DECISIONS` (75), `SELECT_ATOMS`
(364), and `SELECT_ROWS` (2003) with provenance. Gate: byte-identical to the
committed `generated.rs` before extending. Then extend to 69 productions and
+21 fixed terminals (76 -> 97 predicates; `TerminalSet`'s u128 still fits).
Treat the `infix_tail` production / `infix` node-kind name mismatch as a
first-class test of the generator — every other pair shares a name. Note that
byte-for-byte reproduction proves agreement on the v0.22 grammar, not
completeness of the extended tables; the migrated corpus parse is the
completeness oracle, which is the control that was missing when 84 follow rows
went missing in 0031's predecessor.

**5. Candidate assembly is a solved problem.** The round-1 assembler applied 57
verbatim anchors, each asserted to occur exactly once, and its output verified
at 128 rules and 20 sections with the frontend contract changing only in the
`[FORM-1]` section. Re-assembling from the fixed delta is the same mechanical
exercise; the round-1 candidate at
`9ca13585dca6668bec9c1b4c219cbb5fc94559bcc29f67d04d52bbb61af5eef4` is
superseded by the fixed delta and should be regenerated, not patched.

**6. Do not redo.** The blocker below is ruled (FN-4's premise is re-keyed to
the operand-derived selected type; TYPE-5's retained-argument class is not
touched, so review finding F3 stays closed). The GRAM-1 shape-kind deviation
below is accepted and is being folded into the delta as a proper site.

## Blocker (round 1 — RULED, retained for history)

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
