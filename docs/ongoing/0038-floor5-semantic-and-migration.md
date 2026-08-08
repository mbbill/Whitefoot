# 0038 — FLOOR-5 semantic path and corpus migration

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS` — 2026-08-08, rounds 9 through 14. Rounds 13 and 14
  are M3c on `task/0038-m3c-inline-fixtures`, awaiting integration: the inline
  fixtures are migrated, `slice_of` no longer demands the arguments A1 deletes,
  and the six ruled residuals are carried out. The library gate is **568 passed
  / 7 failed**, from 319 / 253 at M3c's base, and the adapter is **383 / 5 /
  14**, with nothing newly failing at any step. `slice_of` cleared both
  `fn1-pos-returned-slice-*-run` as well, confirming the shared root by
  measurement. Of the seven remaining, two are activation-gated and five are
  owned elsewhere — see "Round 14"'s closing table. One new finding:
  `slice_of_keeps_nonflat_element_arguments_in_the_op1_domain` has no v0.23
  expression, with four probes and their controls recorded.
- **Status (rounds 9–12):** Round 9's blocker is **closed**: the
  requires-block `let` has a legal v0.23 form again, the same pass admits the
  infix spelling [FN-8] requires (`8838150`, `8ccd4d8`, `7e80d92`), and round
  10's copy-gate finding is **closed** by judging [FN-8]'s "own copy value" on
  the derived type (`2ccbf4a`, `96735a5`), now with corpus coverage
  (`6b0dd43`). Round 8's findings 1–3 and finding 4's cause are **closed** by
  round 12: ten dispositions carried out, five cases restated so their recorded
  rule fires again, two citations moved, two needing nothing after the
  branch-scope compiler fix. Each branch measured 28 → 16 and 28 → 19 failures
  alone; the composed figure is measured after this integration rather than
  inferred from the two.
  **Two cases remain deliberately failing, each for a recorded reason.**
  `fn2-neg-eeq-implicit-type` cannot be restated: [DIAG-1] requires FN-2 at the
  `call` node for a user-generic call while the compiler cites TYPE-5 for
  missing, wrong-count and wrong-kind arguments alike — a spec/compiler
  discrepancy that stops the affected work.
  `fn8-neg-requires-eeq-payload-enum` was earlier read as sharing an OP-1/OWN-1
  **precedence defect** with `op1-neg-eeq-payload-enum`; **round 12 refuted that
  reading** with a four-cell affinity matrix, so no ordering defect exists. Its
  open question is different and sharper: FN-8's subset pass rejects a `move`
  operand in a requires clause, so `eeq` on a payload-carrying enum may have no
  legal v0.23 spelling there at all. That pair of rejections is to be measured
  before the case is disposed. See "Round 9" through "Round 12".
- **Authority:** owner approval 2026-08-07 and the 2026-08-08 rulings
  (`governance/APPROVALS.md`), including the canonical-renderer ruling; the
  amended delta `governance/spec-evolution/spelling-relief-candidate.md`
- **Owner / workspace:** exec-0038j (round 9) / `/Users/bytedance/do_not_scan/wf0038-r9`
  on branch `task/0038-floor5-semantic-and-migration`; exec-0038k (round 10) /
  `/Users/bytedance/do_not_scan/wf0038-r10` on branch
  `task/0038-conformance-dispositions`, based on round 9's tip `efb5242`
  (resolved from `git log`, not relayed). The two branches are disjoint in
  files: round 10 touches `compiler/src/resolution/`,
  `compiler/src/bin/migrate/`, six case files, and five manifest rows (two
  verdicts, three docs), none of which is a requires-block case. Integration
  order does not matter.
- **Base revision:** 55ff3ff (main), already the merge base; no rebase was owed
  — verified, not assumed: `git merge-base HEAD main` equals main's tip
- **Dependency:** 0036 (grammar path + pins green at 69 productions)

## Goal

The second half of FLOOR-5's atomic activation, split from 0036 on the
first executor's recommendation because the 69-production verifier green is
a real integration boundary:

1. Semantic path — TYPE-5 derivation replacing the deleted annotations,
   OP-2 operand-derived row selection, GIVE-1's derived delivery type
   (a contract inversion in check/control.rs's check_let and matches.rs,
   not a deletion), if_stmt/value_if checking into the existing checked
   Bool-match (see 0036's verified condition (a)), GRAM-6's three new
   rejections, FN-4's re-keyed discharge premise at calls.rs and
   catalog.rs.
2. Corpus migration — 1353 targ deletions, 1748 let annotations, 257 Bool
   matches to if/else with mandatory else-if flattening, ~384 infix
   respells; scripted in scratch, every file passing the branch compiler's
   parse and FORM-2 canonical audit; conformance in the same change with
   no verdict meaning changed.
3. The four conformance cases 0036's condition (b) requires
   (gram6-neg-bool-scrutinee-match, gram6-neg-empty-else,
   gram6-neg-unflattened-else-if, give1-neg-empty-delivery-set), each
   asserting both the cited rule and that the citation lands on the if
   construct.
4. Evidence: both gates exit 0 (direct codes), adapter comparison, and the
   owner REVIEW PACKET (diffstat, ten representative before/after excerpts
   covering every transform class, verdict-meaning statement).

The migrated corpus parse is the completeness oracle for the extended
grammar tables — the verifier triple is not.

## Updated on 0036's completion (lead, 2026-08-08)

Take over branch `task/0036-floor5-grammar-and-migration` (the assembly
and table work lands at commit 5c12646; verify the tip yourself);
do not start a new branch. Already done there: the v0.23 candidate
(SHA-256 `a92b45138c82c3d19dc2f0bfdfe2d04b5571ccc898d6427c9661bf0903b2918e`
as of commit 5c12646 — recompute with `shasum -a 256` rather than trusting
this line; the digest previously written here was fabricated by the lead
and matched no commit),
grammar tables at 69 productions / 84 decisions / 97 terminal predicates,
identity pins at the candidate path, and a repaired
`TerminalPredicate::index()` (the external predicates moved 68..75 →
89..96 after the fixed inventory grew; the old range collided
`Fixed(Minus)` with `Literal`).

Scope additions since the card was written:
- **The lexer operator form** moved here from 0036: `scanner.rs` dispatch,
  the `b'-'` guards as precedent, `classifier.rs`. One existing test
  asserts operator tokens do NOT form; it is now wrong by design and must
  be REWRITTEN against the new behaviour, never deleted.
- **Migration figures, re-measured on the corrected 420-file basis**
  (`tests/conformance` + `tests/programs`; the `tests/codegen` holding
  corpus and `research/experiments` are frozen and excluded): 1588
  deleted-class type arguments, 102 retained-class, 2003 let annotations,
  262 `True()` arms, 378 arithmetic respells, 519 four-comparison
  respells, 207 `ilt`/`igt` (which stay named calls under ruling O1), 406
  `check` statements. Every figure in the delta's §5 carries the command
  that reproduces it; re-run rather than trust.
- **Tooling trap**: `grep` here is ugrep 7.5.0 and silently returns 0 for
  `-oE '(^|[^A-Za-z0-9_.])name<'` — no error, no warning. Use a
  `sed 's/^/ /'` line prefix with a plain character class, and cross-check
  counts with Python `re`.

## Definition of done (lead ruling, 2026-08-08)

The branch finishes GREEN EXCEPT exactly three activation-gated checks,
which cannot pass before the owner approves the candidate bytes and are
not to be weakened or silenced: `spec::tests::path_and_version_label_agree`,
`spec::tests::computed_identity_is_the_approved_digest`, and
`whitefoot-spec`'s `recorded_chain_ends_at_the_embedded_specification`.
The activation commit closes all three at once. `ACTIVE-SPEC:` is an owner
approval record: writing one to make a gate green is forbidden.

### `main` is RED while this batch is live, and this is the exact set

Stated plainly because it is a real cost, not a footnote. The batch landed on
`main` at `8df0e29` with `make check` exit 2 and the library at **568 passed /
6 failed**. Since two of the six are activation-gated by the ruling above,
**`main` cannot be green until the owner approves the candidate bytes**, so
integrating before activation was a lead choice that traded a working gate for
visible progress. Whoever reads a red `main` during this window should know it
was expected, and — more importantly — should still be able to detect a NEW
breakage.

The mechanism is the one this batch already invented: **compare the failure SET
by name, never the count.** Any name outside this list is a real regression
regardless of whether the total went up or down.

| test | owner |
|---|---|
| `spec::tests::path_and_version_label_agree` | activation-gated |
| `spec::tests::computed_identity_is_the_approved_digest` | activation-gated |
| `driver::…::compiler_independent_negative_cases_keep_their_semantic_rule` | its case's violation died with the deleted bytes — NOT a citation defect |
| `semantic::tests::result_construction_…` | shares `x-give-result-aggregate`'s cause |
| `semantic::tests::borrows::general_borrows_…` | capability gap, `RegionsAndBorrows` |
| `semantic::tests::slices::slice_value_matches_…` | capability gap, `OwnershipJoin` |

Conformance adapter alongside it: **Pass=384 Fail=4 Skip=14** after round 15,
the four being `own3-pos-outlives-store` (capability gap, `RegionsAndBorrows`),
`x-give-result-aggregate` (a positive wrongly rejected TYPE-5),
`fn8-neg-requires-eeq-payload-enum` (may have no legal v0.23 spelling in a
requires clause), and `own5-neg-slice-value-match` (capability gap,
`OwnershipJoin`, and it is hiding a negative). Round 15 repurposed the fifth,
`fn2-neg-eeq-implicit-type`, onto live FN-2 content, so it passes rather than
merely disappearing.

**Correction, 2026-08-08, reached independently by the lead and the executing
unit and merged here.** This table first attributed
`driver::…::compiler_independent_negative_cases_…` to the
citation-by-callee-class defect, and by implication `fn2-neg-eeq-implicit-type`
with it. **Both were wrong, and this correction matters more than an ordinary
one**, because the table is the mechanism for telling a real regression from an
expected failure while `main` is red: an entry naming a cause that cannot
produce it makes the next reader either wait for a fix that will never clear
it, or read a genuine change as expected. Measured:
`fn2-neg-implicit-instantiation.wf` now reads `let a = 40_i32 + 2_i32;` and
exits 0, and `fn2-neg-eeq-implicit-type`'s `return eeq(left, right);` exits 0 —
the migration respelled both violations out of existence, so no citation fix
could reach either, and the citation defect being fixed in round 15 while both
still failed is what proved it. Both belong to round 8's finding-2 class.
Only `fn2-neg-eeq-implicit-type` was then repurposed onto live FN-2 content and
passes; **the driver-test entry in the table above is unaffected and still
fails for the recorded reason**, so the two must not be read as one disposition.

No known-failures file, no gate exception, no machinery — a list in the live
record that dies with the record. Adding a mechanism that lets a red gate pass
is exactly the shape this project forbids, and it would outlive the reason for
it.

## Round 9 (exec-0038j, 2026-08-08) — round 8's finding 4 closed; one new blocker

Three commits. Infix is checked at every expression position the grammar has,
and the adapter's failures fall 52 → 28. The 24 that went were all the infix
defect; the 13 that remain are a different one, reported below rather than
absorbed.

### The enumeration, which is the actual deliverable

**Eleven positions take a bare `expr`, not nine.** Extracted from the v0.23
candidate's fenced grammar blocks by matching `expr` as a whole word on every
right-hand side, so `borrow_expr` and the `expr_stmt` left-hand side cannot
inflate it:

| position | infix reachable | was checked |
|---|---|---|
| `ordinary_let_rhs` | yes | yes |
| `propagate_let_rhs` | yes | yes |
| `set_stmt` | yes | yes |
| `return_stmt` | yes | **no** |
| `check_stmt` | yes | yes |
| `claim_stmt` | yes | yes |
| `give_stmt` | yes | yes |
| `match_stmt` scrutinee | yes | yes |
| `value_match` scrutinee | yes | yes |
| `if_stmt` condition | yes | yes |
| `value_if` condition | yes | yes |

The round-8 brief's list of nine **omitted the `if_stmt` and `value_if`
conditions**. Both are reachable — a Bool-producing infix is exactly what an
`if` condition wants — and both were already checked, so the omission cost
nothing this time; it is recorded because the list was offered as the
enumeration and was not one.

`expr_stmt := call ";"` takes a `call`, so infix cannot be written there. It is
excluded by the grammar, not by observation.

**Independently corroborated**: `compiler/src/syntax/grammar/generated.rs`
carries exactly 11 `GrammarNode::new(…Production::Expr…)` entries. Two methods
— the spec's grammar text and the compiler's generated tables — reach eleven.

### `90781eb` — the fix

Root cause, and it is not "infix resolution is missing": ten positions route
through `check_expression`, which has read infix since `3af8478`. `return_stmt`
routes through it too. What broke was two **structural queries** that run before
it and read the `expr` node with `tree.only_child`, and [GRAM-5]'s
`expr := atom infix_tail?` is the one alternative with two children.

The two sites are reached under **complementary** conditions, so each needed its
own proof:

- `check_return_implicit_read` ([TYPE-7]) returns early unless the result mode
  is `own`. This is the site round 8's reproduction hit.
- `complete_borrow_expression` ([OWN-14]) is reached only when the result mode
  is a borrow. Proven separately: a borrow-returning function with an infix
  return expression failed internally, while its control — the same function
  returning a plain atom — cites FN-1 cleanly. An infix can never produce a
  borrow, so FN-1 is the correct outcome and the internal failure was hiding it.

Both ask which of `atom infix_tail? | call | construct` an expression is written
as. Infix is none of the three, so the answer is "no such child" rather than a
failed tree. One shared `sole_expression_child` beside `only_child` answers it
once. A sweep confirms these were the only two: `only_child` has eight callers
and the other six take `item`, `requires_entry`, `stmt`, or a wrapper, never an
`expr`; `Production::InfixTail` has exactly two consumers.

### `bfbe43c`, `435ccb4` — the tests, verified to discriminate

The eleven positions are a table, so a position cannot be missed by fixing
whichever test happened to fail. Two assertions each: the infix checks
completely, and — stronger, because a position that merely stopped failing the
tree could still be skipping the judgment — the same source rewritten to
disagree on its second operand must cite TYPE-5 at exactly that operand. All
eleven do. Plus the borrow-result FN-1 test above, and a backend test that
returns an arithmetic and a comparison result and consumes both, since the
semantic gate says nothing about lowering and this shape had never reached the
backend.

**Not assumed to be gates — measured.** With the fix reverted and the tests
kept, exactly three fail, each reporting
`CompilerFailure { failure: InvalidCanonicalTree }`, and the table names
`return_stmt`. The backend trap guard was checked the same way: entering its
branch on purpose fails the test.

### The three oracles

Both baselines were recomputed in a worktree at `102dfbe` rather than relayed,
and both reproduce round 8's recorded numbers exactly.

1. **Library SET: 4 removed, 0 added.** 300 passed / 266 failed →
   308 / 262. The four are all corpus-embedding tests
   (`compiler_independent_scalar_cases…`,
   `compiler_independent_nominal_data_cases…`,
   `buffers::compiler_independent_borrowed_pool_tree_executes`,
   `contracts::protected_fn4_cases_discharge_only_the_closed_table`). The four
   new tests all pass; 566 + 4 = 570 reconciles.
2. **Library STAGE: 0 moved earlier, 0 moved later, 0 changed within a stage**,
   over the 262 failing in both runs. A targeted fix should move nothing it did
   not fix, and it moved nothing.
3. **Adapter: 335/52/14 → 359/28/14.** 24 resolved, **all 24 at
   Semantics/Compiler**; 0 newly failing; of the 28 still failing, 0 moved
   earlier and 0 changed verdict or stage at all.

**Two brief counts corrected.** The baseline carries **37**
`InvalidCanonicalTree` adapter failures, not 38. And the infix defect accounted
for **24** of them, not all of them — the remaining 13 are the separate cause
below. Both figures are reproducible from the two adapter logs.

### Blocker — the requires-block `let` has no legal form

**All 13 remaining `InvalidCanonicalTree` adapter failures are requires-block
cases**: `ent3-pos-s4-requires-fact`, `fn3-neg-requires-member`, the seven
`fn8-*-requires-*`, `fn8-trap-requires-false`, `x-base64-rfc-vectors-run`, and
`x-requires-output-capacity-run`. Not the infix defect, and not fixed here.

`validate_requires_let` (`compiler/src/semantic/check/requires.rs:78`) requires
a `Mode` child and then a `Type` child, failing `InvalidCanonicalTree` when
either is absent. Those are the two parts of the `let` annotation that v0.23's
A3 deletes. Four controls isolate it:

```
requires { let ok = ilt(a, 8_u64); … }             # InvalidCanonicalTree
requires { let ok: own Bool = ilt(a, 8_u64); … }   # Parsing/Source [GRAM-4]
requires { check a <= 8_u64 else trap "…"; }       # exit 0 — infix is fine here
requires { check ilt(a, 8_u64) else trap "…"; }    # exit 0
```

The middle two are the point: **there is now no way to write a requires-block
`let`** — with the annotation it no longer parses, without it the checker fails
internally — and infix inside a requires block already works, so this is a
distinct defect that happens to share a symptom.

**Why it was not fixed here.** Restoring the restriction needs the binding's
mode and type derived from the initializer under [TYPE-5] instead of read off
the deleted annotation, and `validate_requires_statement` runs *before*
`check_statement`, where no derivation exists yet. Choosing between reordering
validation after checking and deriving separately is an unwritten design
decision, not a mechanical repair, so it belongs to whoever owns the semantic
path's TYPE-5 derivation item rather than to this unit.

### Validation

- `make -C compiler check`: **exit 2** (`$?`, not through a pipe), lib
  **308 passed / 262 failed** — the same exit and the same failure classes as
  the baseline, with 0 tests regressed.
- `make check`: **exit 2**, failing at the same compiler step. Earlier stages
  pass: spec append-only, spec archive integrity at 23, conformance coverage
  **128/128 rules, 0 uncovered**.
- `cargo clippy --all-targets -D warnings` exit 0; `cargo fmt --check` exit 0.
- The three activation-gated checks remain red by the definition of done above,
  and nothing was written to make them green.

## Round 12 (exec-0038m, 2026-08-08) — round 8's findings 1-3 disposed; one blocker

*Renumbered from "round 10" at integration: this unit ran on a parallel branch
and chose the next number it could see, which rounds 10 and 11 on the sibling
branch had already taken. Its own commits and evidence are unchanged. The
identity was likewise `exec-0038k` on both branches and is distinguished here.*

Four commits. Nine of the ten cases round 8 left are resolved and none newly
fails; the tenth is refused with a reproduction rather than written to match
the compiler.

**The count is TEN, not nine.** Recomputed from the adapter's own list rather
than from the brief: the 28 baseline failures are 13 requires-block cases (the
sibling branch's defect), the 10 disposed here, and 5 outside this scope.
Finding 1 is one case, finding 2 is two, finding 3 is six, finding 4 is one.

```
$ grep -c "^  Fail " base-adapter.log
28
```

### `f22951f` — two findings had one cause, and it was a compiler defect

`ScopeBuild` opened a lexical scope for `loop_stmt`, `region_stmt`, and `arm`,
and **nothing for `if_stmt` or `value_if`**, so every branch `let` declared into
the enclosing block. The migration turned 260 Bool matches into conditionals,
which is why this surfaced now.

[GRAM-4] is why this construct is the exception — it hangs both `stmt*`
sequences off one node, so a walk keyed on child productions cannot separate
them:

```
$ grep -n '^if_stmt     :=' governance/spec-evolution/kernel-spec-v0.23-candidate.md
153:if_stmt     := "if" expr "{" stmt* "}" ("else" (if_stmt | "{" stmt* "}"))?
```

[TYPE-6] admits both rejected programs verbatim, and still forbids the shadow:

```
$ grep -n 'Disjoint expired lexical scopes may reuse' …v0.23-candidate.md
239:… A nested lexical declaration may not shadow an entry live at that
    declaration. … Disjoint expired lexical scopes may reuse an ordinary value
    or label spelling …
```

Four sources, two of them controls. Sibling branches reusing a spelling, and a
branch binder followed by the same spelling in the enclosing block, both now
resolve. The **`match`-arm spelling of the first program** — separate
productions, same program — must reach the same answer and does. A branch
binder shadowing a **live** enclosing binder must still be rejected, and is:
per-branch scopes could easily have hidden it. With the fix reverted and the
test kept it fails on the first source, so it discriminates.

This closes finding 4 and **dissolves one member of finding 3**:

| case | brief said | actually needed |
|---|---|---|
| `ent5-pos-join-keeps-common-bound` | fix the cause | fixed; compiles, exit 0 |
| `ent2-neg-expired-spelling-inherits-nothing` | restate OP-4 → TYPE-6 | **refused**; cites its recorded OP-4 again, row unchanged |

Writing that TYPE-6 row would have recorded a compiler defect as a normative
expectation. This is the whole point of verifying the citation against the
specification instead of against the compiler.

### `c899b83` — finding 1, and the class closed by rule

`form2-neg-noncanonical-ws` keeps the migrated line with its four-space
indentation restored, and rejects the recorded verdict:

```
$ ./compiler/target/debug/whitefootc --emit-llvm \
    tests/conformance/cases/form2-neg-noncanonical-ws.wf ; echo "exit=$?"
CanonicalSource/Source [FORM-2]: CanonicalIssue { … }
exit=1
$ grep -n 'Every nonempty physical line begins with exactly two ASCII spaces' …
60:Every nonempty physical line begins with exactly two ASCII spaces for each
   enclosing brace block.
```

Round 8's "leave it unmigrated" disposition does not hold: unmigrated it cites
GRAM-4, because a v0.22 annotation is itself a v0.23 grammar error.

**The exclusion is now derived from the manifest by rule.** §2 "Canonical form"
is exactly FORM-1 through FORM-7 plus the LEX-1 policy rule no case can assert,
so a case whose required verdict cites a `FORM-*` rule is a case about bytes.
**Measured: 16 of the 401 case files, not the one the brief guessed.**

```
$ for f in tests/conformance/cases/*.wf; do whitefoot-migrate --check "$f"; …
total=401 kept=16 changed=0 unchanged=377 refused=8
```

One invocation per file so the exit code is read from `$?`, never through a
pipe. The rule covers all 12 FORM-family members of the retired 20-name hand
list, adds the one it missed, and adds the three FORM-7 cases — which rendering
does preserve, since terminal interiors keep their bytes, but whose subject is
still a literal's spelling. Splitting the family to spare them would be a
per-rule hand list again, which is the defect. The other 8 of the old 20 are
refused because they do not parse, which needs no list at all.

### `6a4b916` — findings 2 and 3, per case

Every verdict observed by running the case; no verdict predicted.

| case | recorded | observed after | disposition |
|---|---|---|---|
| `x-typ-bool-cmp-result-as-int` | TYPE-5 | TYPE-5 | source restated, row unchanged |
| `type7-neg-implicit-read` | TYPE-7 | TYPE-7 | source restated, row unchanged |
| `x-typ-match-foreign-variant` | TYPE-6 | TYPE-6 | source restated, row unchanged |
| `op1-neg-eeq-payload-enum` | OP-1 | OP-1 | source restated, row unchanged |
| `own1-neg-match-move-through-borrow` | OWN-1 | OWN-5 | **row moved**, source unchanged |
| `x-match-give1-wrong-type` | TYPE-5 | GIVE-1 | **row moved**, source restated |
| `ent2-neg-expired-spelling-inherits-nothing` | OP-4 | OP-4 | nothing needed |
| `fn2-neg-eeq-implicit-type` | FN-2 | accepts | **refused, blocked** |

The spec text behind each, by `grep -n` on the candidate:

- **220** [TYPE-5] "the right-hand side of `set p = e;` must produce exactly
  `own T` … a different right-hand-side mode or type is a hard error citing
  TYPE-5". A binder's type is now its initializer's, so nothing can disagree
  there; a `set` target's type still comes from a written declaration. The case
  commits `a == b` to a written `i32` field.
- **243** [TYPE-7] "A borrow-mode or box/arena binding used where a value of its
  referent type T is expected is a hard error citing TYPE-7". A declared
  parameter type is still such a position, so the case passes the borrow to
  `takes(value: p)`.
- **196** [GRAM-6] "a `match` whose scrutinee has type `Bool` is a hard error
  citing GRAM-6 at the scrutinee `expr` node". That is why the Bool scrutinee
  had to go; the case now matches a source enum against a prelude `Some` arm and
  earns TYPE-6 `ForeignMatchVariant`.
- **259** [OWN-1] "tag-only enums (every variant nullary …) copy on use; all
  other values … are affine". A payload-carrying enum is affine, so the deleted
  type argument left bare operands earning OWN-1 first. Moving both operands
  restores OP-1's domain judgment.
- **267** [OWN-5] "Content reached through any borrow may never be moved: `move`
  requires a place rooted at an own-mode binding." OWN-1 defines when a move is
  legal and never states this. The row was wrong before the migration — it is
  main's single red case.
- **200** [GIVE-1] "a delivering `give` whose exact mode or type differs from an
  earlier delivering `give` of the same initializer is a hard error citing
  GIVE-1 at the later `give_stmt`". The recorded 2026-08-08 amendment already
  ruled this rewrite; its `traps` row, which the compiler cited as EFF-2, is
  leftover incidental content.

**Where this diverges from the brief, and why.** The brief ruled three of these
as manifest citation changes: TYPE-7 → FN-1, TYPE-6 → GRAM-6, OP-1 → OWN-1. In
each the compiler's citation is literally conforming — FN-1 at **424** does own
an unreachable statement, and `type7-neg-implicit-read`'s trailing `return
unit;` genuinely is one. But in each the cited violation is *incidental or
earlier-firing* while the case's own concern is still expressible in v0.23, so
moving the row would have deleted that concern's negative coverage silently —
the failure mode this batch exists to remove, and the case the ruling of record
already covers ("restate minimally so the recorded rule fires again, keeping the
witness where possible"). The two rows that did move are the two whose concern
genuinely died with the deleted bytes.

### The OP-1/OWN-1 reading, checked rather than assumed

A concurrent executor working `fn8-neg-requires-eeq-payload-enum` observed that
it and `op1-neg-eeq-payload-enum` both reach `Reject(Some("OWN-1"))` where their
rows want OP-1, and read the pair as an OP-1/OWN-1 **precedence defect** in the
checker. That reading was considered here and **does not survive**. It is
recorded because two units disagreed about one case.

The affinity matrix, four sources, each run alone:

| enum | operands | verdict |
|---|---|---|
| tag-only (copy) | bare | **exit 0**, accepted |
| tag-only (copy) | moved | OWN-1 `MoveOfCopy` |
| payload (affine) | bare | OWN-1 `BareAffineUse` |
| payload (affine) | moved | **OP-1 `InvalidOperation`** |

Both OWN-1 kinds are the two halves of one OWN-1 sentence, and the checker
tracks operand affinity in both directions rather than preferring a rule:

```
$ grep -n 'Every other bare `place` expression of affine type is a hard error' …
259:… tag-only enums (every variant nullary …) copy on use; all other values …
    are affine. … Every other bare `place` expression of affine type is a hard
    error (write `move p`), and `move p` on a copy value is a hard error …
```

A payload-carrying enum is affine by that classification, so the migrated bytes
carried **two independent violations**, and OWN-1's was real. Which of two gets
reported is settled by [DIAG-1], and it settles against the precedence reading
from both directions:

```
$ grep -n 'Two or more simultaneously established post-resolution semantic' …
652:… rejections whose immediate offending source premise is the same use of the
    same canonical node are one rejection event, and that event cites the
    established rule whose definition appears first in this specification …
    The order among rejection events at distinct nodes is implementation-defined.
```

The two premises here are at **distinct** nodes — OWN-1 at the operand `atom`
(bytes 152-156, `left`), OP-1 at the operation call (bytes 148-174) — so the
order is implementation-defined and the compiler's choice conforms. And had they
been at one node, OWN-1 is defined at line 259 against OP-1 at line 325, so the
same-node rule would mandate **OWN-1** as well. There is no ordering the
specification demands and the compiler withholds.

The row was therefore never changed. The disposition is the restatement above:
give the file exactly one violation by moving both operands, and OP-1 becomes
forced rather than traversal-dependent. **The general lesson is the fragile
shape, not a checker bug** — any negative case whose file carries two violations
at distinct nodes has an implementation-defined citation, which is what made
this row look moved at all.

Not investigated, and stated as inference from the controls above rather than
from that file: if `fn8-neg-requires-eeq-payload-enum` is also
`BareAffineUse` on a payload-carrying enum with bare operands, it has this same
real cause and the same one-line fix, not a precedence defect. It belongs to the
other executor.

### Blocker — the FN-2 diagnostic path cites TYPE-5

`fn2-neg-eeq-implicit-type` is **unchanged and still accepts**. It writes
`eeq(left, right)` bare on a tag-only enum, which v0.23 makes legal by design.

FN-2 (**426**) does retain negative content, so this is not the specification
finding the brief said to stop on. Its two surviving classes are the region-free
`targ` requirement — already covered by two live passing cases, so restating
onto it adds nothing — and "instantiation arguments are always explicit", which
is this case's own concern. Reaching that is blocked:

```
$ grep -n "The cited rule is the rule selected by the callee's class" …
658:… [FN-2] for a user-generic call … a missing, wrong-kind, wrong-count, or
    wrong-domain argument … uses `SourceNode` at the `call` node and that
    node's complete source extent.
```

Three reproductions on `fn identity<T: Int>(value: own T)`, with the correct
call as the control:

```
identity<i32>(value: 1_i32)       exit 0                     — control
identity(value: 1_i32)            [TYPE-5] at the call node  — want FN-2
identity<i32, i64>(value: 1_i32)  [TYPE-5] at the call node  — want FN-2
identity<7>(value: 1_i32)         [TYPE-5] at the `targ`     — want FN-2 at the call
```

So user generics work, and every FN-2 argument defect is cited TYPE-5, one of
them at the wrong node. This is the gap already recorded as the `pending` reason
on `fn2-neg-implicit-instantiation` ("does not yet implement … its FN-2
diagnostic path"), now with a reproduction. A spec/compiler discrepancy stops
the affected work, so nothing was written for this case.

### Also found, outside this scope and not touched

- **Three more positive cases reject after the migration**, the same class as
  finding 4 and not in the ten: `x-give-result-aggregate` (want Run, reaches
  TYPE-5 `TypeMismatch`), `fn1-pos-returned-slice-inputs-run` and
  `fn1-pos-returned-slice-const-run` (want Run, reach FN-2 `InvalidOperation`).
  Main's lane has one red case, so these are migration-caused and presumed
  defects by the same ruling. The branch-scope fix does not clear them.
- **`every_canonical_corpus_file_re_renders_to_itself` is red, before and after
  this round**, and not one of the three activation-gated checks. It asserts
  `underived.is_empty()`, which the standing ruling that 20 cases stay at v0.22
  makes unreachable. Measured both ways: before this round 420 files as 399
  round-tripped / 0 non-canonical / **21** underived; after, 399 / 1 / **20** —
  strictly better, since the FORM-2 restatement now derives. Its
  `DELIBERATELY_NONCANONICAL` is also still a two-name hand list, so the gate
  cannot pass until whoever owns it reconciles the two rulings. Not changed
  here: rewriting a gate's assertion to make it green is exactly what an
  executor may not do.

### Validation

- `make -C compiler check`: **exit 2** before and after (`$?`, not through a
  pipe), lib **308 → 309 passed**, **262 → 262 failed with a byte-identical
  failing set** (`diff` of the two sorted name lists is empty). The one added
  pass is the new resolver test.
- `make check`: **exit 2** before and after, failing at the same compiler step.
  Earlier stages pass: spec append-only, spec archive integrity at 23, corpus
  structure 18/18, conformance coverage **128/128 rules, 0 uncovered** —
  unchanged by the two moved citations.
- `cargo fmt --check` exit 0; `cargo clippy --all-targets -D warnings` exit 0.
- **Adapter: 359/28/14 → 368/19/14**, in two measured steps. The scope fix alone
  gives 361/26/14 (`ent2`, `ent5` leave). The case work gives 368/19/14
  (`form2-neg-noncanonical-ws`, `op1-neg-eeq-payload-enum`,
  `own1-neg-match-move-through-borrow`, `type7-neg-implicit-read`,
  `x-match-give1-wrong-type`, `x-typ-bool-cmp-result-as-int`,
  `x-typ-match-foreign-variant` leave). **Nine left, zero arrived**, by set
  `diff` rather than by count. The 19 remaining are the 13 requires-block cases,
  `fn2-neg-eeq-implicit-type`, and the 5 out-of-scope failures.
- The three activation-gated checks remain red by the definition of done, and
  nothing was written to make them green.

## Round 8 (exec-0038g, 2026-08-08) — M3b ran; four things need a ruling

Three commits. The corpus is migrated and the failure set shrinks for the
first time in this batch. Four findings stop short of closing it, and each
carries a reproduction.

**Handoff state found, corrected before work.** The brief said no worktree
held the branch; `/Users/bytedance/do_not_scan/wf0038-r7` did, with the
`reject-err2-nonexhaustive` restatement uncommitted. Both resolved
themselves mid-orientation — the branch was committed (`70859f3`), rebased
onto `e8054fe` and its worktree removed while this round was reading — so
nothing was destroyed and no rebase was owed. Verified rather than assumed:
`git merge-base main task/0038…` equals main's tip.

### `a7cb4e4` — the two structural assertions, as a library gate

`compiler/src/syntax/parser/finalize/tests/corpus_shape.rs`. Round 7 was
right that a bin cannot reach `FinalizedTopology`; a library test is also a
standing gate rather than a one-shot. The detector ships with its own
controls — both forbidden forms, the enum `match` and `else if` chain that
replace them, an `else` holding an `if` *plus another statement* (which
cannot be flattened and is not the defect), the empty `else` that is a
different [GRAM-6] clause, and a source that reaches no tree.

It landed **red on exactly one file** and went green with the migration,
which is the evidence that it fires.

**One limitation, stated rather than papered over.** A tree carries no
types, so a Bool `match` is detected by its arms naming `True`/`False`.
`x-typ-match-foreign-variant.wf` has a Bool scrutinee and a `Some` arm, so
the assertion cannot see it — the adapter does (below). Widening the
criterion needs types, not a better pattern.

`type5-neg-match-non-enum.wf` matches `True()` against an `i32`; the
2026-08-08 ruling left it deliberately. It is named in the test and asserted
to still be present, so neither a second such file nor its disappearance
passes unnoticed.

### `060a0f0` — the corpus migration

310 files. **The 20 exclusions were measured, not assumed**: `--check` over
all 420 one file at a time refuses exactly 20, and that set is identical to
the ruled list, 20 of 20. Re-running the tool over the result changes 0
files, and the corpus round-trip gate reports all 400 round-tripping.

Measured class figures are in the delta's §5 (`96862aa`) and reconcile to
the site: 1575 written type arguments in the 400 plus 12 in the excluded 20
is the 1587 the old command now returns, and A1's 690 plus C1's 886 is 1576,
one more because `fn2-neg-implicit-instantiation.wf` writes `iadd.trap(...)`
with no argument to delete. 261 `True()` arms, 260 conditionals, one ruled
survivor. **The drop from the recorded 312/691 is the `reject-err2`
restatement, proven by running the tool on that file's pre-restatement
bytes: exactly 1 changed file and 1 argument list.**

### The three oracles

1. **Failure SET: 20 removed, 0 added.** Lib 280 → 300 passed, 286 → 266
   failed. 19 are tests that `include_bytes!` a corpus case — the lib does
   embed 96 corpus files across 19 sources, which is why a corpus migration
   moves it at all — and the 20th is the corpus assertion above.
2. **Stage oracle: 0 moved earlier, 17 moved later, 0 changed within a
   stage**, over the 266 failing in both runs. Parsing 92 → 61, Resolution
   1 → 10, Semantics 0 → 8. Predicted before the run and confirmed.
3. **Adapter: 116/271/14 → 335/52/14.** Main's lane was recomputed in its
   own worktree rather than relayed: **386/1/14**.

### Finding 1 — a conformance case the migration destroys (NOT landed)

`form2-neg-noncanonical-ws.wf` is **reverted to its committed bytes** and
needs a ruling. Its entire content under test is an isolated four-space
indentation; canonical re-rendering removes it, and the migrated file
**compiles cleanly (exit 0) where its manifest row demands a FORM-2
rejection**. Changing protected conformance evidence is not an executor's
call, so it is left unmigrated and reported.

This falsifies one sentence of the 2026-08-08 ruling — "every byte-level
case is in the untouched 13 … the set turned out to be disjoint from the set
needing edits." This is a byte-level case that both parses after the pre-pass
and needs an edit.

**A minimal restatement is verified and ready**: take the migrated line and
restore its four-space indentation, i.e.

```
fn main() -> own unit pure {
    let a = 1_i32;
  return unit;
}
```

which rejects `CanonicalSource/Source [FORM-2]` — the recorded verdict,
observed rather than inferred. Unmigrated it instead cites GRAM-4, so
neither leaving it nor migrating it is correct; only the restatement is.

### Finding 2 — two cases whose concern died with the deleted bytes

Both now **accept where their manifest demands a rejection**, and neither is
a tool defect:

- `x-typ-bool-cmp-result-as-int` (TYPE-5). Its violation was
  `let v: own i32 = ieq<i32>(a, b);` — an annotation disagreeing with the
  right-hand side. A3 deletes the annotation, so there is nothing left to
  disagree. Under v0.23 the concern **cannot be expressed at all**, because
  a binder's type is its initializer's by construction.
- `fn2-neg-eeq-implicit-type` (FN-2). **Untouched by the migration** — it
  already writes `eeq(left, right)` bare. v0.22 rejected the missing type
  argument; v0.23 deletes that argument by design, so it is now legal.

The delta already anticipates the class — "the error classes that lived only
in deleted bytes die with their bytes" — but retiring or restating a
conformance case is protected evidence and needs owner agreement plus an
approval-ledger entry. Nothing was changed.

### Finding 3 — six cases reject under a different rule

Verdict kind preserved, citation moved. `type7-neg-implicit-read`
TYPE-7 → FN-1; `ent2-neg-expired-spelling-inherits-nothing` OP-4 → TYPE-6;
`x-typ-match-foreign-variant` TYPE-6 → GRAM-6; `x-match-give1-wrong-type`
TYPE-5 → EFF-2; `own1-neg-match-move-through-borrow` OWN-1 → OWN-5;
`op1-neg-eeq-payload-enum` OP-1 → OWN-1. Same masking class the ruling
handled for the 20, now on cases that parse. Plus one positive,
`ent5-pos-join-keeps-common-bound`, newly rejecting TYPE-6
`DeclarationCollision`.

### Finding 4 — infix has no checker path in `return` position

**38 of the 52 remaining adapter failures are
`Semantics/Compiler: InvalidCanonicalTree`**, an internal compiler failure
rather than a source rejection. Reduced to a minimal reproduction with a
control that distinguishes the cause — the same operation in a `let`
initializer is checked normally:

```
fn eq(a: own i32, b: own i32) -> own Bool pure {
  return a == b;                    # Semantics/Compiler: InvalidCanonicalTree
}
```
```
let c = a == b;                     # accepted, exit 0
let c = left == right;              # own Bool: Semantics/Source [OP-1], correct
let c = a + b;                      # in a pure fn: Semantics/Source [EFF-2], correct
return a + b;                       # Semantics/Compiler: InvalidCanonicalTree
```

[OP-1] (ii) infix resolution landed at `3af8478` for the `let` path; the
`return` path did not get one. The migration did not cause this — it
exposed it, because the corpus now writes infix where it wrote named calls.

### Validation

- `make -C compiler check`: **exit 2** (read from `$?`), lib
  **300 passed / 266 failed**.
- `make check`: **exit 2**, failing at the same compiler step. Earlier
  stages pass: repository invariants, spec append-only, spec archive
  integrity at 23, conformance coverage **128/128 rules, 0 uncovered**.
- `cargo clippy --all-targets -D warnings` exit 0; `cargo fmt` applied.
- Corpus round-trip gate: 400 round-tripped, 20 underived (exactly the
  ruled 20). Its `underived.is_empty()` assertion is falsified by the
  ruling and is a **fifth open item** — the round-5 brief predicted
  "418 round-tripped, 2 non-canonical, 0 underived" before the 20 were
  ruled out. It was left alone rather than relaxed.

### The candidate did not move, and no pin was re-keyed

The round-8 brief said landing §5's figures "changes the candidate's bytes,
so re-assemble it and re-key all three digest pins". **Measured false.** No
§5 content appears in `kernel-spec-v0.23-candidate.md` (checked for seven
distinctive strings, all zero), and the candidate hashes to
`ab257aa65874c4e6de167189b97cf706b5ca0045ccab86fdb54da83e2ba613da` both
before and after the edit — which is already what all three pins record
(`compiler/src/spec.rs`'s byte array, `tests/conformance/runner.py:80`,
`spec/derivation/derivation-ledger.md:841`). Re-keying would have churned
three correct pins.

## Claim corrections (exec-0038, 2026-08-07)

Three facts in the handoff paragraph above did not hold as written when
this task was claimed. The lead has since repaired that paragraph on main
(`6b1dd18`), so it now reads correctly; what follows is the record of what
was found, not a live complaint.

- **Branch tip `b0da5f8` did not exist** in this repository
  (`git cat-file -t` fatal). The real tip of
  `task/0036-floor5-grammar-and-migration` was `2ec8248`, carrying exactly
  the three described commits; the reflog shows `b0da5f8` was never a
  value of this ref. Taken over at `2ec8248`.
- **The candidate hash was neither cited value.** The committed candidate
  hashed to `935b9538…` — round 5's own reported bytes — not the
  `bde4a9ef…` the card then cited, which matches no commit in this
  repository. It was also *stale*: assembled before main's `32e2af4`
  extended the [OP-1] (iii) anchor, so it still carried the doubled "in
  this version" clause round 5 reported and left literal. Applying the
  delta's corrected (iii) replacement — the only delta change since
  assembly — gives `a92b4513…`.
- **The round-5 blocker is settled by the definition of done above**, which
  rules candidate-stage pinning a recognized state with exactly three
  checks left red until the owner's activation commit.

## Candidate bytes reconciled with 0036 round 6 (exec-0038, 2026-08-07)

The lead directed this branch to take exec-0036d's round-6 commit
(`5c12646`, reachable only from the tag `0036-round6-candidate`) for the
corrected candidate and the three digest pins. **Recomputed rather than
relayed, that commit's four substantive files are byte-identical to what
this branch already carried**, so nothing was owed:

| file | tag vs this branch |
|---|---|
| `governance/spec-evolution/kernel-spec-v0.23-candidate.md` | identical |
| `compiler/src/spec.rs` | identical |
| `tests/conformance/runner.py` | identical |
| `spec/derivation/derivation-ledger.md` | identical |

Only `docs/ongoing/0036-floor5-grammar-and-migration.md` differed, and it
is imported here verbatim from the tag so 0036's round-6 history survives
the tag's deletion.

**This is a real independent cross-check, not a coincidence.** Round 6
re-assembled the whole candidate from the corrected delta by script; this
task reached the same bytes by applying the single corrected (iii) anchor
to the round-5 assembly. Two different methods, one digest —
`a92b45138c82c3d19dc2f0bfdfe2d04b5571ccc898d6427c9661bf0903b2918e`,
recomputed with `shasum -a 256` from this worktree and from the git object
at HEAD. That is stronger evidence for the bytes than either method alone.

One process note against this branch: the candidate fix and its three pins
landed inside the lexer commit rather than as their own commit, which is
why a reader diffing `cacfa26` alone sees the unfixed bytes. The content is
correct and verified; the commit boundary was not as clean as it should
have been.

## Progress (round 1, exec-0038, 2026-08-07 — handed back at a clean boundary)

**Landed.** The front end is complete and green for the v0.23 grammar; the
semantic path and the corpus migration are not started.

1. `cacfa26` — claim, with the three brief corrections above.
2. `7f2da74` — the lexer operator form. `TokenKind` gains `OperatorForm`
   and the four compound comparisons; `scanner.rs` dispatch gains `==`,
   `!=`, `<=`, `>=` ahead of their single-byte prefixes and one
   `b'+' | b'-' | b'*' | b'/' | b'%'` arm after the existing `->`,
   negative-numeric and comment-prefix guards; `operator_form` takes the
   maximal `[a-z]*` suffix. `classifier.rs` admits exactly the sixteen
   `infix_op` rows via a new `FixedTerminal::is_operator_form`, and a
   suffix outside that list is a terminal-membership rejection owned by
   the new `TerminalIssueOwner::Gram1` — a source issue, not a compiler
   failure. A lone `!` stays a raw lexical defect.
3. `1cbafea` — [OP-1] site (i)'s twenty op-column respellings in
   `resolution/catalog.rs`, plus (iii)'s derived-set consequence and the
   syntax-fixture migration.

**The two tests that asserted operator tokens do not form were rewritten,
not deleted**, and three positive tests were added:
`operator_forms_take_their_maximal_lowercase_suffix`,
`compound_punctuation_beats_its_single_byte_prefixes`, and
`operator_suffix_membership_admits_exactly_the_infix_op_list` (which
carries the `/sat` and `%sat` near misses). The single-byte enumeration in
`every_single_top_level_byte_has_a_controlled_lossless_outcome` gains the
five operator bytes and still excludes `!`.

**Round 4's predicted parser risk does not materialize — verified, not
assumed.** A probe (added, read, removed) shows `let b = a + a;` derives an
`Expr` node with two children whose extent is the complete `a + a`, with
`InfixTail` and `InfixOp` beneath it; `if`/`else` and a `value_if` also
derive. The compiler's node kinds *are* `Production`s and `Expr` already
wrapped every expression in v0.22, so the "`infix` node must span the
complete `expr`" requirement is already met and no parser change is owed.
`syntax::` is wholly green (67 tests).

**Gate states, exit codes read from `$?` with no pipe.**

- `make -C compiler check`: **exit 2** — lib 256 passed / 271 failed.
- `make check`: **exit 2**; its earlier stages pass (repository invariants,
  spec append-only, spec archive integrity at 23, conformance plumbing OK)
  and it fails at the compiler stage on the same 271.
- `whitefoot-grammar CANDIDATE CANDIDATE`: **exit 0** — **69 productions,
  84 decisions, 97 terminal predicates**, against the corrected bytes.
- `cargo test --bin whitefoot-grammar-tables`: **ok** — the tables still
  derive from the candidate after the [OP-1] (iii) repair, which is the
  expected result because that repair moves prose, not EBNF.
- `make conformance-run`: **exit 2** — adapter **Pass=116 Fail=271
  Skip=14** against main's 386/1/14.
- The three activation-gated checks are exactly the predicted three and
  nothing else: `spec::tests::path_and_version_label_agree`,
  `spec::tests::computed_identity_is_the_approved_digest`, and
  `whitefoot-spec`'s `recorded_chain_ends_at_the_embedded_specification`.
  `recorded_identity_is_the_computed_identity` **passes**, which is the
  independent check that the re-keyed digest is right.

**Migration figures re-measured, not trusted** (Python `re` with a true
lookbehind, on the 420-file basis, string literals blanked). Basis 420;
deleted-class type arguments **1588**; retained-class **102**; let
annotations **2003**; `True()` arms **262**; four-comparison sites
**519**; `ilt`/`igt` **207**; `check` statements **406** — all reproduce
the card exactly. Arithmetic respells reproduce **378** by the delta's own
`name<` command; counting `name(` as well gives **379**, and the single
extra site is the finding below.

## Verdict-meaning breaks found — these need a ruling before migration

The task forbids changing any verdict's meaning and says to stop out and
list any case that would. Four do. None is mechanically migratable.

1. **`fn2-neg-implicit-instantiation`** — the extra arithmetic site above.
   Its whole body is `let a: own i32 = iadd.trap(40_i32, 2_i32);` and it
   asserts reject FN-2 for *a table operation with no explicit
   instantiation argument*. After A1 no table operation carries a written
   type argument at all, and [OP-2] (g)'s own analysis records that
   "FN-2's missing-type-argument judgment goes with the written argument
   it was about". The premise is deleted, so the migrated bytes
   (`let a = 40_i32 + 2_i32;`) are a **valid program** — the verdict
   inverts. FN-2 still governs *user* generics, so the case can be
   re-expressed onto a user-generic callee, but that changes its subject
   and is an owner/lead decision, not an executor's.
2. **`x-typ-bool-cmp-result-as-int`** — asserts reject TYPE-5 for binding
   a Bool comparison result to an `own i32` let. A3 deletes the let
   annotation, so the disagreement it tests has no spelling and
   `let v = a == b;` simply derives `own Bool`. Verdict inverts.
3. **`x-match-give1-wrong-type`** — asserts reject TYPE-5 because a
   `give` does not deliver "the let's declared mode type". The new
   [GIVE-1] derives the type *from* the delivery set, so this premise is
   gone; the nearest surviving clause is disagreement between two
   delivering `give`s, which is a different rejection needing a rewritten
   case.
4. **`gram9-neg-nested-call`** — `imul.wrap<i32>(iadd.wrap<i32>(…), …)`.
   GRAM-9 still rejects a non-atom infix operand, but there is no
   parenthesization surface in which to spell a nested infix, so the case
   needs re-expression (for example a retained `ilt(…)` call in an infix
   operand position) rather than a substitution.

The other nine cases the scan flagged are safe: `op1-neg-ieq-bool` and
`op1-neg-ineg-unsigned` reject on operand domain, which [OP-2] (f) and (g)
preserve word for word after the argument is dropped.

## Progress (round 2, exec-0038, 2026-08-07) — FORM-2 blocker cleared

`5895526` clears the item round 1 named as the successor's first blocker,
so the front end is now complete for v0.23 end to end: lex, classify,
parse, finalize, and FORM-2 canonical rendering.

**What the blocker actually was.** `NodeRecord` held one brace pair,
assigned at `finalize/engine.rs` from the *last* `{` and `}` the node
owned, so an `if` with a braced `else` kept only the else block. Braces
now pair in source order into a second optional pair (`else_open` /
`else_close`) that only `if_stmt` and `value_if` ever use; every other
block-bearing production still owns exactly one pair, because a nested
block always belongs to a nested production node. `inside_body` tests both
pairs, so else-block children get their format depth.

**The join line.** `} else {` is a rendering no v0.22 production produced.
It follows the `requires` block's `} {` precedent: suppress the break
after a close brace that joins a continuation. An `else if` chain reaches
the same rule through the `else` terminal the outer node owns while the
nested `if_stmt` owns the second block, so the outer node has one pair
plus `has_else`.

**Evidence.** `if_else_renders_its_join_line_and_indents_both_blocks`
pins the exact canonical bytes for four shapes — else-free `if`, braced
`else`, an `else if` chain, and a `value_if` initializer — through
`only_these_trivia_bytes_render`, which additionally asserts that removing
*any single* trivia byte makes the source non-canonical. Full lib run
after the change: **257 passed / 271 failed**, with the failure set
byte-identical to before it (no regressions, no accidental fixes); `cargo
fmt --check` and `cargo clippy --all-targets -D warnings` both exit 0.

The migration's FORM-2 canonical audit can now run, which was the reason
this had to come first.

**One property is structural and must stay that way.** An else-position
`if_stmt` begins *after* the then-block's closing brace, so it lies inside
no brace pair of its parent and `inside_body` leaves it at the outer
`format_depth`. FORM-2's "an else-position `if` is never rendered as a
nested introducer line" therefore needs no special case: nothing in the
depth computation mentions `if_stmt`, and `has_else` is read only when
suppressing the break after a close brace. A three-deep `else if` chain is
pinned in the test for exactly this reason — add a special case and depth
accumulates, indenting each arm one level deeper, and that fixture fails.

The two-pair design was independently derived and converged on by a second
executor before the collision was detected; that is corroboration, not
review.

## Two method habits worth copying (lead ruling, 2026-08-07)

Named here because they are cheap, general, and each caught something on
this branch that a looser method would have missed.

**Pin a new rendering to the byte, not to acceptance.** `} else {` is a
line no v0.22 production produced, so asserting merely that it *parses*
or *audits clean* would pass for a renderer that put the brace anywhere.
`only_these_trivia_bytes_render` asserts the source is canonical **and**
that deleting any single trivia byte makes it non-canonical, which fixes
the join line exactly. Use it for any layout rule with no precedent.

**Diff failure sets, never failure counts.** A count is not evidence: one
regression cancelling one fix leaves the total unchanged. Every run on
this branch wrote its sorted failure names to a file and `comm`-diffed
them against the previous run, which is how "257 passed / 271 failed" was
shown to be exactly the new test and not a swap. It is also how round 1
caught that respelling the catalog turned nine passing tests into failing
ones — invisible in the total, obvious in the set.

## Round 3 blocker (exec-0038, 2026-08-07) — A3 makes `None()` untypeable

**This needs an owner/lead ruling and it moves the candidate bytes, so it
stops the semantic path here.** [TYPE-5]'s rewritten text asserts that an
`ordinary_let_rhs` is "always self-typed", listing "constructions name
their nominal" as one of the four reasons. That claim is false for the
nullary prelude variant constructions, and A3 deletes the annotation that
was carrying them.

Reproduction, `compiler/src/backend/tests/resource_enums.rs:140`:

```
let abandoned_none: own Option<buffer<u8>> = None();
```

Migrated by A3 this is `let abandoned_none = None();`. Nothing in that
right-hand side supplies `buffer<u8>`: `None()` has no payload to derive an
element type from, and `Option` is not in [TYPE-5]'s retained-argument
class, so no written argument is admitted either. The binding's type is
not reconstructible, uniquely or otherwise.

**The compiler's current channel is exactly the one A3 removes.**
`check_construct` (`semantic/check/expressions.rs:820,843`) resolves the
prelude `Option` and `Result` constructors — ordinals 5, 6, 11 and 13 —
by `let Some(CheckedType::Nominal(nominal)) = expected else { reject
TYPE-5 }`. It reads the expectation and **never consults written type
arguments**, so `None<buffer<u8>>()` does not rescue the site either; that
spelling is rejected today for the same reason. `let_stmt` is the only
construct position whose expectation came from an annotation — `set` uses
the target type and call arguments use declared parameter types — so A3
removes the sole supply for this class.

**Measured scope**, basis and method as in the round-3 count block below.
Corpus: **1** annotated let-RHS prelude construction
(`tests/programs/generic_nominals.wf:86`, a `Some`). Compiler inline
fixtures: **4** (`backend/tests/resource_enums.rs:139,141,144` and
`semantic/tests/options.rs:21`), one of which is the `None()` above. The
count is small; the rule is not, because after A3 `let x = None();` has no
legal spelling anywhere.

**Why an executor cannot settle it.** The three repairs are a language
choice, and two of them move normative bytes that three digest pins name:

- (a) Add the prelude variant constructors to [TYPE-5]'s retained-argument
  class and make `check_construct` honour written `targs`. This is the
  repair the delta's own rationale already argues for — the retained class
  exists "because no operand can supply them", which is exactly true of
  `None()`. Spelling becomes `None<buffer<u8>>()` and
  `Some<u8>(value: v)`. Moves the candidate.
- (b) Derive the nominal from the payload. Handles `Some`/`Ok`/`Err` and
  **cannot** handle `None()`, so it does not close the gap.
- (c) Keep an expected-type channel for the `let` initializer. Contradicts
  [TYPE-5]'s "no binder's type depends on ... an expected type" in the same
  paragraph.

(a) is the only coherent one. It is a delta change, so it re-keys
`a92b4513…` and the three pins.

Everything above this line on the branch is verified and unchanged:
candidate digest `a92b45138c82c3d19dc2f0bfdfe2d04b5571ccc898d6427c9661bf0903b2918e`
(recomputed, and the `spec.rs` byte array decodes to it), rebase onto main
`e810ce5` clean over 13 commits, and `make -C compiler check` **exit 2**
with **257 passed / 271 failed**, the failure set captured for `comm`
diffing.

### Expectation sweep (exec-0038, 2026-08-07) — COMPLETE, and it found a second hole

The lead ruled (a) and asked for the generalization to be applied
exhaustively before the migration: enumerate every consumer of an expected
type and say what supplies it after A3. Done, and **the sweep is complete
rather than a sample**, because the positions are enumerable from the
grammar rather than found by search.

**Why it is complete.** `check_atom` takes no expectation, and under
[GRAM-9] a `construct` is not an `atom` — `atom := literal | "move" place |
place | borrow_expr`. So a construction can only appear as a complete
`expr`, and the `expr` positions are exactly the five below. There is no
sixth place for this class of failure to hide.

| consumer | site | supplier after A3 | |
|---|---|---|---|
| `return_stmt` | `control.rs:167` | `function.result` [FN-1] | **survives** |
| `set_stmt` | `control.rs:348` | `target.ty()` [SET-1] | **survives** |
| `propagate_let_rhs` | `results.rs:47` | built from the let's expectation; [ERR-3] derives the payload from the callee signature | plumbing, not a hole |
| `ordinary_let_rhs` | `control.rs:560` | the deleted annotation | **REMOVED** |
| `give_stmt` | `control.rs:307` | `GiveContext.expected`, itself the deleted annotation | **REMOVED** |

**The `give` row is new and no brief names it.** Under the rewritten
[GIVE-1] the delivery set derives the binding's type, so the *first*
delivering `give` has no expectation by construction — the same hole as
the `let`, in a second position. Sites, on the migration basis:

```
BASIS | PREP | grep -oE '[^A-Za-z0-9_](give|return)\s+(Some|None|Ok|Err)\('
```

- `let` RHS: **1** (`tests/programs/generic_nominals.wf:86`)
- `give`: **2** (`tests/conformance/cases/x-give-result-aggregate.wf:4,7`)
- `return`: **98** — supplier survives, untouched
- `set`: **0**

**This is why the sweep had to precede the amendment.** Drafting from the
original finding alone would have covered `let` only; `give` would have
surfaced during migration and forced a *second* re-key of the candidate
and all three digest pins. The amendment must cover both positions in one
pass. The ruled repair already does, since making the prelude variant
constructors carry written arguments mandatorily removes the dependence on
an expectation in every position at once — which is the argument for
making it mandatory rather than optional-where-needed.

### Sequencing consequence: the rejections and the migration are one batch

Measured while looking for a unit that could proceed around the blocker:
**there is no independent one left.** Each remaining semantic item is
coupled to the corpus migration, so landing any of them alone converts
passing tests to failing — the same shape as round 1's catalog respell.

- **GRAM-6's Bool-scrutinee rejection**: `compiler/src` inline sources hold
  **87** `True() =>` arms across **12** files (`backend/tests*.rs` ×8,
  `semantic/tests*.rs` ×3, `lowering/tests.rs`). Every one becomes a
  rejection the moment GRAM-6 lands, so it must land together with their
  migration to `if`/`else`.
- **FN-4's re-keyed premise** expects the discharge body `return p0 +sat
  p1;`, which cannot be spelled until infix resolution exists, so it is
  coupled to [OP-1] (ii).
- **[OP-1] (ii) infix resolution** is what clears the nine tests round 1's
  catalog respell converted to failing; the catalog is already landed, so
  this half is owed, but its oracle is the migrated sources.

So the remainder is one atomic batch — semantic path plus migration plus
the conformance cases — and its first transform class (A3's let-annotation
deletion) is what the blocker above gates. That is why this round stops
here rather than banking a partial semantic change: a half-landed
rejection is indistinguishable from a regression in the gate.

**The blocker gates 5 sites, not the batch.** A successor may migrate all
four transform classes and leave exactly the five prelude-construction
sites failing, as the visible and reproducible consequence of the open
ruling. That is the recommended shape if the ruling is slow: it is honest,
it isolates the gap to five named sites, and it weakens nothing.

## Round 3 counts (exec-0038, 2026-08-07) — independently confirmed

All nine migration figures reproduce by two methods that share no failure
mode: the delta §5 ugrep commands, and Python `re` with a true lookbehind
(`(?<![A-Za-z0-9_.])`) with the deleted-class alternation re-derived from
the v0.22 op table at run time rather than hard-coded. Basis **420**,
deleted-class **1588**, retained **102**, let annotations **2003**,
`True()` arms **262**, arithmetic **378**, comparison **519**,
`ilt`/`igt` **207**, `check` **406**. Also 610 `.wf` files repo-wide with
**0** free-standing `if` tokens.

Two things the recomputation settled. The 378/379 discrepancy is the
single argument-free table-op call in the basis,
`tests/conformance/cases/fn2-neg-implicit-instantiation.wf:3` — which is
ruling (1)'s subject, so that ruling's premise is unique rather than one
of a class. And ruling (3) was amended before use: its case matches on a
Bool scrutinee, so a rewrite keeping the `match` form would assert GIVE-1
and earn GRAM-6; the enum-scrutinee form is now recorded at `e810ce5`.

## Round 4 (exec-0038b, 2026-08-08) — the ruling applied; handed back at the boundary

One commit, `9ab2728`. The round-3 blocker is closed and everything it
gated is re-derived rather than carried forward.

**The amendment.** The prelude variant constructors join [TYPE-5]'s
retained-argument class, mandatorily and in every position. It adds **no
site** — it edits replacement text inside the existing TYPE-5 site, exactly
as the three pattern-7 corrections do — so the totals stay **64 sites across
34 rules**, and §7's recount is re-derived and says so. §1's version-header
paragraph also gains a sentence; it is the proposed header rather than a §3
anchor/replacement pair, so it has never been counted as a site.

**Candidate SHA-256 `ab257aa65874c4e6de167189b97cf706b5ca0045ccab86fdb54da83e2ba613da`**
(was `a92b4513…`). Re-assembled by a script that reads the replacement bytes
out of the delta's own blockquotes, and whose unwrap procedure was **first
validated by reproducing the committed candidate's two lines from the
previous delta revision** — the round trip is byte-exact, which is what says
the assembler is faithful. The whole-file diff against the committed
candidate is exactly the two amended lines.

**Three pins re-keyed, including the byte-array form.** `compiler/src/spec.rs`
`ACTIVE_KERNEL_SPEC_HASH` (the 32-byte array at line 60 — the one an earlier
grep missed, re-keyed here from the hex by script rather than by hand),
`tests/conformance/runner.py`, and `spec/derivation/derivation-ledger.md`.
`spec::tests::recorded_identity_is_the_computed_identity` **passes**, which is
the independent check that the array decodes to the new digest.
`computed_identity_is_the_approved_digest` keeps pinning v0.22's owner-approved
`b133b793…` and stays red by design.

**Nothing structural moved, verified rather than assumed.** §2's two EBNF
blocks re-hash to MD5 `00f6095415ba43440367b87d94f06a3e` and
`cfd44788e1b76e4017271f8e639f2308`, and [EX-1]'s program block to
`814fdabade0cea99e3879bd5fdc6f892` — the three properties §7 records for
exactly this purpose. The three frontend-contract ranges are unmoved
(`[FORM-1]`..`## 4. Types` 26879B both before and after; the other two
byte-identical to v0.22). The candidate self-verifies at 128 distinct rules
and 20 sections with zero surviving `let` annotations.

**Two findings the ruling's premises did not contain.**

1. **The expectation sweep's completeness proof was short.** It enumerated
   five `expr` positions; v0.22's grammar has **nine** (lines 149–161). The
   four missed are `check_stmt`, `claim_stmt`, `match_stmt` and
   `value_match`. This does not change the ruling — it strengthens it. Three
   of the four never had an expectation to lose, so a prelude construction
   was *already* untypeable there in v0.22, and the corpus proves it: two
   conformance cases assert exactly that rejection
   (`x-enum-option-context-free-constructor`,
   `x-enum-result-context-free-constructor`, `match Some(value: x)` as
   scrutinee). A repair scoped to "positions A3 breaks" would have fixed
   `let` and `give` and left those two alone. The mandatory form fixes all
   nine because it stops the rule quantifying over positions at all.
2. **The amendment adds no grammar and no new spelling, and resolves a v0.22
   internal divergence rather than choosing new behaviour.** `construct :=
   TYPEID targs? "(" fieldinit_list? ")"` already admits the arguments;
   [FN-2] already calls instantiation arguments for a "PRE-1 nominal generic
   parameter" always explicit; and [TYPE-6]'s "construction and matching never
   consult an expected nominal type" already forbade the channel
   `check_construct` was using. The citation is pinned to **TYPE-5 at the
   complete `construct`** because that is what `generic_substitution`
   (`semantic/check/generics.rs:545,550,556`) already reports for source
   generic nominals, so the two classes report identically instead of
   diverging by declaration source — and that is what preserves the two
   conformance cases' verdict, rule and node.

**O1 restated** as "mechanism ruled, deferred for batch hygiene" and moved out
of the Open list, with the four binding conditions written as conditions on
these bytes: `<` and `>` reserved; and FORM-2's attachment sets, O5, O9 and
EX-1's bytes each annotated as **closed over this version's operator set
only**, with the re-run each owes stated. O1's stale "56 sites" is corrected
to the measured **207**.

**A fourth ugrep trap, measured.** A negated bracket class followed by a
literal `(` (either `\(` or `[(]`) silently matches **zero**, and whether it
does depends on the rest of the pattern: `[^A-Za-z0-9_.](None|Some|Ok|Err)[(][^)]*[)]`
matches, and the same pattern with a trailing ` *=>` does not. There is no
defensive pattern-shape rule; the construct is unusable here. Every new
figure uses `grep -oP` with a true PCRE lookbehind, cross-checked against
Python `re`. Recorded in §5 with the measurement table.

**All nine migration figures re-measured on this branch after the rebase and
reproduce to the digit** (420 / 1588 / 102 / 2003 / 262 / 378 / 519 / 207 /
406). Script at `/Users/bytedance/do_not_scan/wf0038x/recount.py`, re-pointed
at the live worktree and extended with the new class; exits non-zero on
divergence.

**A fifth transform class the migration now owes: 103 sites.** 416 tokens of
the four generic-prelude constructor spellings, of which **313 are `arm`
patterns and migrate by no rule** (`arm := TYPEID "(" fieldbind_list? ")"
"=>" …` has no `targs` child; the nominal comes from the scrutinee). The
remaining **103** are constructions and **0** already carry written
arguments. The split by position is total and closes exactly: 98 `return` +
2 `give` + 2 `match` scrutinee + 1 `let` RHS + 0 `set`. **101 are rewritten
and 2 are pinned bare** — the two scrutinee sites are the negative cases
above, and writing their arguments would invert them; they keep verdict,
rule and node, and only their `doc` reason is re-worded.

**Ruling (4) re-checked after the delta moved, not carried.** `ilt` and `igt`
do survive in the amended candidate — the op-table row reads
`` | `==` `!=` `ilt` `<=` `igt` `>=` | all int T | ``, and [OP-2]'s comparison
paragraph and [ENT-3]'s comparison-origin clause both still name them. So
`gram9-neg-nested-call`'s repair may use a surviving named call, and the
brief's assumption holds.

**Gate states, exit codes read from `$?` with no pipe.**

- `make -C compiler check`: **exit 2** — lib **257 passed / 271 failed**.
- `make check`: **exit 2**; earlier stages pass (repository invariants, spec
  append-only, spec archive integrity at 23, conformance plumbing OK).
- `make conformance-run`: **exit 2** — adapter **Pass=116 Fail=271 Skip=14**
  against main's 386/1/14.
- `cargo test --bin whitefoot-grammar-tables`: **exit 0**.
- `whitefoot-grammar` against the amended candidate: **exit 0** —
  **69 productions, 84 decisions, 97 terminal predicates**, unchanged.

**The failure SET is byte-identical to round 3's** — `comm` against
`/Users/bytedance/do_not_scan/wf0038-baseline-failures.txt` shows zero added
and zero removed. That is the check that a prose-only amendment plus a digest
re-key regressed nothing and fixed nothing, which a count alone could not say.

### Why round 4 stops here

The remainder is the atomic batch, and its first transform class is what the
now-closed blocker gated. Opening it costs more than a round: the semantic
path, ~3800 corpus sites across five transform classes, 87 `True() =>` arms
across 12 compiler test files, four repurposed and four new conformance
cases, and the review packet. Landing a fraction of that leaves the gate
indistinguishable from a regression, which is the shape round 3 named. The
amendment, by contrast, is complete, self-verifying, and is what every
remaining unit is keyed to — so it is a real boundary rather than a pause.

### One finding that changes the migration's shape — read this first

**§5's "All migration is printer-driven" describes a capability the compiler
does not have.** `compiler/src/syntax/parser/finalize/canonical/` is an
**auditor**, not a renderer: `audit_canonical` is the only public entry point
and it answers "are these bytes canonical?", while `bytes_match` and
`gap_matches` compare against the source rather than emitting anything. There
is no code path that produces canonical bytes from a tree, so a migration
cannot "run the printer" — as written it must get every byte of spacing and
indentation right itself, including the `} else {` join line that no v0.22
production produced and the else-if flattening.

The repair is small and is the recommended shape rather than a blocker.
`build_gap_styles` already computes the canonical layout decision per token
boundary (`GapStyle::{Inline, Break, Blank}`), and `bytes_match` already
carries the indentation rule off `format_depth`. A renderer is those two
facts plus the token spellings — a modest addition over existing machinery,
not a new design. Building it first turns the migration into: a **textual**
pre-pass that need only produce something that *parses*, then a render pass
that makes the bytes canonical by construction. That is both far cheaper than
a layout-exact textual transform and the only form that satisfies O1's
binding condition 4, because the next version re-runs it rather than re-doing
it.

## Round 5 (exec-0038d, 2026-08-08) — the renderer and its gate; handed back at that boundary

One commit, `bcb639f`. It discharges items 1 and 2 of the 2026-08-08 canonical
renderer ruling. Item 3, the migration tool, is not started.

**The renderer shares the auditor's rules rather than matching them.** The
ruling's condition was that the two agree by sharing code. The gap descriptor
is factored out of `gap_matches` into `canonical_gap(style, depth, left,
right) -> CanonicalGap`, where `CanonicalGap` is `{newlines, spaces}` — every
FORM-2 gap has that shape. The auditor compares source bytes against it and
`render.rs` emits it, so there is one layout rule with two consumers, not two
implementations. `build_gap_styles` is reused unchanged.

**Rendering has no source verdict**, and that asymmetry is deliberate:
`audit_canonical` asks whether given bytes are canonical and can answer no,
while `render_canonical` is handed a finalized tree and produces the bytes that
tree denotes, so `RenderOutcome` carries only `Complete`, `ResourceFailure`
and `CompilerFailure`. It borrows the bundle rather than consuming it, because
it publishes no capability.

**The gate**, per the hygiene rule that a tool ships attached to a caller:
`compiler/tests/canonical_corpus.rs` runs inside `make check` and holds every
corpus file to FORM-2 — a canonical file re-renders to itself, a deliberately
non-canonical one renders to different bytes, and whatever goes in, what comes
out is canonical and idempotent. The last clause is the one the migration
rests on, so it is asserted for every file rather than only the canonical ones.

**118 of the 420 corpus files round-trip byte-exactly today.** The other 302
carry v0.22 spellings and do not derive under the v0.23 grammar, which is what
the migration fixes. That 118 is a real byte-exact control on the renderer
available before any migration, which is what the round-4 brief asked for; the
round-4 brief's expectation of a ~420-file control before migration does not
hold on this branch, because the v0.23 grammar is what the un-migrated corpus
fails to parse.

**The deliberately non-canonical class is two, not one**, enumerated from the
manifest as the round-4 brief instructed: `form2-neg-noncanonical-ws` and
`x-form-form2-tab-indent`, both `expect.rule == FORM-2`. They are named in the
gate rather than skipped by pattern so a third cannot appear unnoticed.

**Gate states, exit codes read from `$?` with no pipe.**

- `make -C compiler check`: **exit 2** — lib **259 passed / 271 failed**.
- `make check`: **exit 2**; earlier stages pass (repository invariants, spec
  append-only, spec archive integrity at 23, conformance plumbing OK).
- `make conformance-run`: **exit 2** — adapter **Pass=116 Fail=271 Skip=14**
  against main's 386/1/14, unchanged.
- `whitefoot-grammar` against the amended candidate: **exit 0** — **69
  productions, 84 decisions, 97 terminal predicates**, unchanged.
- `cargo fmt --check`, `clippy --all-targets -D warnings`, `cargo doc`: **0**.

**The lib failure SET is byte-identical to round 4's** — `comm` against
`/Users/bytedance/do_not_scan/wf0038-baseline-failures.txt` shows zero added
and zero removed. Passing moved 257 → 259, which is exactly the two new
renderer tests. The candidate digest and all three pins were recomputed rather
than relayed and hold at `ab257aa6…`, including the 32-byte array in
`spec.rs`.

**One consequence for the definition of done, reported rather than absorbed.**
The new corpus gate is a *fourth* check that is red until the migration lands,
where the ruling names three. It does not change the observable gate state
today: `cargo test --all-targets` stops at the failing lib target, so the
integration test does not run under `make check` yet and the failure set is
unchanged. It becomes active the moment the lib goes green, which is the
migration's completeness oracle and is where it belongs. The branch's finish
line is still "green except the three activation-gated checks" — the fourth
closes with the batch, not with the owner's approval.

### Three measurements that de-risk the migration tool

Cheap here, expensive for a successor to rediscover.

1. **The 20-spelling rename map is derivable, not hard-coded.** Extracting the
   [OP-1] op column from v0.22 and from the candidate by the method
   `catalog.rs`'s own `extract_operation_families` test uses gives 83 rows
   each, row-aligned, differing in exactly **20** cells — the twenty
   respellings, `iadd.wrap → +wrap` through `imul.sat → *sat`. The migration
   tool should zip the two specifications rather than carry a table, which is
   the same discipline `whitefoot-grammar-tables` already follows.
2. **The infix respell is mechanically total.** Of the **897** respell sites
   (378 arithmetic + 519 comparison, reproducing the delta's figures exactly),
   exactly **one** has an operand that is not a GRAM-9 `atom`:
   `gram9-neg-nested-call.wf:3`, which is already ruling (4)'s subject. Every
   other operand is an IDENT, a place, a literal, or a negative literal.
3. **`deref(...)` is a place, so it is a legal infix operand.** `pbase :=
   IDENT | "deref" "(" place ")"`. A naive "does an operand contain `(`" scan
   reports **33** non-atom operands; 32 of them are `deref(p)` or
   `deref(p).field` and are fine. Do not treat that scan's output as a
   blocker list.

### Why round 5 stops here

The renderer is what everything else was keyed to, and it is complete,
self-verifying against 118 real files, and green on its own terms. The
migration tool is the next unit and is large: five transform classes over
~3800 sites, of which the Bool-match-to-`if`/`else` flattening is the only one
needing real structure rather than token rewriting. Starting it with the
budget left would have produced a half-built tool that a successor must first
understand and then probably discard — the shape rounds 2 and 4 of 0036 both
named. The render pass makes that tool much smaller than it looks: the
pre-pass may emit any layout that parses, so it never computes a byte of
spacing or indentation.

## Round 7 (exec-0038f, 2026-08-08) — M2: GRAM-6, the `if` path, and the fixtures

One commit, `a79a676`. M2 is complete. M1's `box_new` blocker is untouched
and still open; it is independent of everything here.

**Three claims in the round-6 hand-back brief did not hold.** Branch tip
`a4d3d33` **does not exist** (`git rev-parse` fatal) — the third fabricated
commit id on this task, and the lead has since adopted the practice of
naming the branch and never an id. "M1 is complete" was **false**: at
`9931e0f` the OP-2 half was still uncommitted working-tree state, and the
worktree holding the branch was **actively being written** (two files
modified between two `git status` runs 90 seconds apart). This round held
rather than racing, and took the branch only after round 6 committed
`9eed20a` and `4da1717` and released. The fixture figure was wrong in both
briefs: not 87 across 12 and not 142 across 20, but **105 Bool matches, 210
arms, 13 files**, cross-checked by True and False counts being equal in
every one of the 13 — the check neither earlier figure had. A `compiler/src`
scope silently drops `compiler/tests/programs/wide_scan.rs`, which holds 36
of the arms. The candidate digest `ab257aa6...` did verify.

### What M2 needed that the brief said was cheap

"Fold `if` into the existing checked Bool match" is true of **lowering** and
false of the front half. `compiler/src/semantic/` contained **zero**
occurrences of `IfStmt` or `ValueIf`, `check_statement` had no arm for one,
and `SemanticRule` had no `Gram6` at all. Adding it moved the definition
rank of every rule after it up by one, because the rank is machine-checked
against the active specification text; GRAM-6 sits at rank 2 there.

The lowering claim itself is now **demonstrated rather than asserted**: a
backend test compiles and runs a program whose every branch is observable,
so a wrong one traps. `check_if` builds its two arms from the same Bool
descriptor the `match` used, so both spellings produce one
`CheckedStatement::Match` over `CheckedEnumType::Bool`.

### The one design trap, found by a compiler failure rather than by reading

[GIVE-1] gives an else-if chain **one** delivery set, belonging to the
chain's binding. A chained `value_if`'s nested node is itself a `ValueIf`,
which `check_statement` has no arm for, so routing the alternative through
it produced `Semantics/Compiler: InvalidCanonicalTree`. The shared body now
recurses with `opens_delivery = false`: only the outermost `value_if` opens
the context and each chained one contributes to it, exactly as a statement
`match` propagates its `give`s. **`opens_delivery` is not "this is a
`value_if`"**, and a successor changing this code should not collapse them.

### Why M2 cannot be judged by a green count, and what judges it instead

Every test touching those fixtures was **already failing** before this
round, on the v0.22 annotations, deleted type arguments and prefix
spellings M3 owns. Migrating Bool matches turns nothing green. Three
oracles were used instead:

1. **The failure SET is byte-identical** — zero added, zero removed against
   the 285-entry baseline recomputed at `4da1717` (the brief's 271-entry
   file predates round 6's commits and is stale). Lib goes 259 -> 271
   passed on the 12 new tests.
2. **No test's failure moved to an earlier stage.** A botched fixture would
   fail at parsing where it used to fail at semantics; across all 285, the
   stage changed for **none**.
3. **Zero surviving Bool-scrutinee matches**, the round-5 ruling's own
   assertion, applied to the Rust fixtures: `True() =>` and `False() =>`
   occur nowhere outside `semantic/tests/conditionals.rs`, whose fixture
   must stay a Bool `match` to prove GRAM-6 rejects it.

### Two findings a successor needs

**Arithmetic is currently unusable on this branch, and it is not M2's.**
The named form is gone — `iadd.wrap` fails resolution with `UnresolvedUse
{ role: OperationCallee, available: [] }` because round 1 respelled the
catalog — while the infix form `a +wrap b` parses and then fails
`Semantics/Compiler: InvalidCanonicalTree`, i.e. the checker has no infix
path yet. So `let b = a +wrap 1_i32;` compiles under neither spelling. That
is [OP-1] (ii) infix resolution, which the round-3 brief predicted "wants
to land together" with the catalog respell and which has not landed. It
accounts for a large share of the 285 and it blocks any new test that needs
arithmetic; the backend test here is deliberately written with Bool
constructors, `set` and `give` only.

**One migration hazard, hit and fixed.** `format!` fixtures spell a
Whitefoot brace `{{` and a placeholder `{name}`. A first transformer pass
doubled the placeholders into `{{name}}`, which would have silently stopped
substituting in three files. It was caught by diffing for that exact
pattern, reverted, and redone by folding `{{`/`}}` to private markers
before matching so a placeholder is never seen. A successor migrating the
420-file corpus does not face this, but a successor touching these Rust
fixtures again does.

### The `box_new` repair (ruling (b), landed `4e68436`)

The blocker round 6 raised is discharged without moving a normative byte.
`box_new(v)` derives its box nominal from the operand, so after A3 a purely
local box names `box<T>` nowhere for the written-type interning pass to
find, and the lookup died with `InvalidResolution` — an implementation
limitation deciding source acceptability, on a control pair differing only
in whether another declaration happened to spell `box<u64>`.

`check_box_new` now records the missed referent and returns a private
`CheckStop::DeferredBoxNominal`; the `&mut self` driver interns the pending
referents and rechecks that one function. Each attempt must intern at least
one new nominal, which bounds the loop, and the signal never reaches a
diagnostic.

**The phase order had to move with it, and that is the part worth knowing.**
`executable_nominal_count` closed *before* `collect_contracts`, which ran
*before* function checking — so a nominal discovered while checking a
function would have landed outside the executable prefix and never been
lowered, producing an IR referencing a nominal that does not exist.
Contracts are collected after function checking now, which is safe because
nothing in the function path reads them: a source contract is rejected as a
generic bound [FN-3]. The prefix closes after the derived boxes and before
the contract metadata. The guard asserting function checking interns
nothing became a guard that it interns only boxes — the invariant it was
actually protecting.

Two regressions: the control pair, which also asserts the derived nominal is
inside the executable prefix, and an executing test that allocates a
`box<Bool>` spelled nowhere, reads it back and releases it.

### [OP-1] (ii) infix resolution (landed `3af8478`)

The finding that gated M3 is closed. Arithmetic and comparison compiled
under **neither** spelling: round 1's catalog respell deleted the named
forms, and nothing had taught the checker to read the infix one, so
`iadd.wrap` failed resolution with an empty candidate set while `a +wrap b`
parsed and died at `InvalidCanonicalTree`. The cause was one line —
`check_expression_in_context` took `only_child`, and [GRAM-5]'s
`expr := atom infix_tail?` is the one shape with two children.

The operator selects the row by its exact bytes, and the row check is
**extracted rather than copied**, so [OP-2]'s operand-derived selection, the
trap site and the checked-error result are decided once for both spellings.

**Five operators needed the `box_new` repair generalized, and that is the
finding worth carrying.** The checked rows `+checked`, `-checked`,
`*checked`, `/checked` and `%checked` produce `Result<T, Overflow>` and
`Result<T, DivError>` over a *derived* `T`, which after A3 nothing writes —
the identical defect the `box_new` blocker named, in a second nominal
family, and equally a compiler failure on valid source. It was invisible
until infix resolution made those rows reachable at all. `prelude_nominal`
now defers exactly as a missed box referent does, so the mechanism is one
`CheckStop::DeferredNominal` over a pending list of both kinds. **A third
family may exist**: the general shape is "a nominal instance named only by a
derived type", and any lookup-only accessor over an intern table has it.

The guard on function checking became the invariant it was really
protecting: no *source* nominal instance is discovered while checking a
function.

### The derived-nominal sweep (2026-08-08) — COMPLETE, and it finds no third site

Two members of this class were found by tripping over them, so this
enumerates it instead. **The enumeration is complete, not best-effort**, and
it is complete because it runs from a closed set: every intern table is a
field of `struct Checker`, so enumerating the 25 fields enumerates the
tables. 94 fallible lookup sites across `semantic/` were then classified.

**The criterion is sharper than "the key is a derived type".** What exposes
a table is *what the interning pass keys off*:

- keyed off a written **TYPE** — A3 deletes the annotation, the key loses
  its only source, and the lookup misses. **Exposed.**
- keyed off a written **CALL** or **DECLARATION** — A3 deletes neither, so
  the interning pass still sees it. **Immune.**

By that criterion the 25 fields partition exactly:

| Table | Key | Verdict |
| --- | --- | --- |
| `box_nominals` | `CheckedType` | **Exposed** — handled, defers (`4e68436`) |
| `prelude_nominals` | `PreludeType` | **Exposed** — handled, defers (`3af8478`) |
| `system_nominals` | `u8` | Immune — interned from the call |
| 7 `*_by_declaration` maps, `constants` | `DeclarationId` | Immune — a declaration the resolver produced; A3 deletes no declarations |
| 12 index-keyed `Vec`s | allocated id / index | Immune — a miss is an internal inconsistency, unreachable from source spelling |
| `pending_nominals`, `resolved`, `tree`, `reject_entailment` | — | not intern tables |

**`system_nominals` is the instructive one.** Its key *is* a derived value,
so the crude criterion would have flagged it. It is immune because
`nominal_instances.rs:305-312` walks every `call` node and interns every
system parameter and result type from the operation catalog — keyed off the
call, which A3 does not touch. Verified empirically: an annotation-free
system program with `args_count`, `cvt` and `exit_status` compiles. The same
pre-pass has `ensure_conversion_result` for `cvt`, which is why `cvt`'s
`Result` never had the defect while the checked arithmetic rows did — in
v0.22 their `Result` was named by the annotation and by nothing else.

**Residual non-deferring sites over a derived key, and why each is outside
the class:** `types.rs:164`, `:190`, `:199` look up `box<T>`, `Option<T>`
and `Result<T, E>` while parsing a **written type node**, so the key has a
written source that A3 does not delete — signature and field types survive
it. They were left alone rather than made to defer, because deferring would
convert a genuine internal-inconsistency detector into a silent repair for
no benefit the class needs.

**Result: exactly two tables were ever exposed, both are handled, and there
is no third.** The `prelude_nominal` deferral is also a general safety net:
any prelude instance a future derived type names now interns on demand,
whatever produces it.

### M3a — the migration tool (landed `16e8432`), and what the corpus run must decide first

The tool is the forced shape: textual pre-pass to *parseable* v0.23, then
parse, then `render_canonical`. The pre-pass walks the compiler's own
lexemes, so a v0.22 spelling inside a string or comment is a single lexeme
the walk never edits — the inline-fixture hazard removed by construction
rather than defended against.

**A3 is the only parse-blocking class.** Measured, not assumed: a v0.22
corpus file lexes cleanly under the v0.23 lexer and fails at *parsing* on
the `let` annotation ([GRAM-4]). A1's type arguments, C1's named operations
and A4's Bool matches all parse today and fail semantically. So deleting the
annotation — with the coupled prelude-constructor rewrite, since the
constructor's arguments are what the annotation carried — restores the parse
gate on its own, and the remaining classes land without reopening it.

Measured on all 420 corpus files with `--check`: **400 parse and render.**

### The 20 that do not, and why it is a ruling rather than a bug

All 20 are `expect: {"kind": "reject"}` in `tests/conformance/manifest.jsonl`
— verified against the manifest, not inferred from their names, 20 of 20.
They are the cases rejected at or before parsing, so a parse-and-render
migration cannot process them **by construction**. Four do not even lex.

**Re-rendering one would silently destroy the case.**
`x-form-form2-tab-indent.wf` exists to be rejected for tab indentation;
canonical rendering fixes the indentation, so the migrated file would no
longer test anything and would still be green. That is the exact class of
silent verdict change the project forbids, and no gate would catch it —
the case would keep passing.

The other 177 reject cases are rejected *after* parsing, parse fine, and
migrate normally, so this is not a property of negative cases in general.
It is a property of the pre-semantic ones.

**M3b therefore cannot be "run the tool over 420 files".** The owner or lead
must rule on the 20: migrate them textually with no render, leave them at
v0.22 spelling with a recorded reason, or restate each case against v0.23.
The choice is a protected-evidence decision, not an executor's.

### M3a, second half (landed `60476ef`) — and where the prelude class really is

Three more classes: [OP-1]'s respells, the de-argumented rows, and the
returned prelude constructors. Respelling **reorders** rather than renames —
`iadd.wrap<i32>(x, y)` becomes `x +wrap y` — and [GRAM-9] is what makes that
a textual transform at all, because the operands are atoms. Respelling is
asked before de-argumenting, since it subsumes it for the rows it covers.

**The round-5 brief put the prelude class in the wrong place, and the
measurement says so.** It recorded that a constructor takes its arguments
from the `let` annotation A3 deletes. Over the corpus that describes **1 of
107** uses: **98 are returned**, 2 given, 2 passed as arguments. The
arguments therefore come from the enclosing function's declared result type
— which A3 does not delete, and which the pre-pass reads textually with no
inference at all. The `let` rule alone wrote **1** constructor; adding the
returned rule writes **99**.

**First trustworthy per-class figures**, produced by the compiler's own
lexer over the 400 corpus files that parse — every earlier figure in this
batch, mine included, was a regex estimate:

| Class | Measured | Brief |
| --- | --- | --- |
| A3 `let` annotations | **1992** | 2003 |
| C1 respells | **886** | 897 (378 + 519) |
| A1 argument lists | **691** | — |
| prelude constructors written | **99** (0 left bare) | 101 of 103 |
| files changed | 312 of 400 | — |

The 691 and the 886 **reconcile the brief's 1588 written arguments**: 1577
of them are deleted by these two classes together, because a respelled row
carries its argument away with the call form it dissolves. That is a units
mismatch in the old figure, not an error in it.

Remaining: A4, the Bool matches. It is the one class that reshapes a tree
rather than respelling tokens, so the renderer cannot rescue a mistake in
it; the algorithm is the one already proven by hand on 105 inline fixtures.

### The 20 pre-semantic reject cases — measured and disposed (ruling 2026-08-08)

The ruling's key point is one neither obvious option had: **leaving them at
v0.22 is not automatically safe either**, because a v0.22 annotation is
itself a [GRAM-4] error under v0.23, so a case rejected at the grammar stage
may cite the annotation's rule instead of its own — the same silent verdict
change, from the other direction. So each case was run against the v0.23
compiler and its cited rule compared to its manifest row. A measurement, not
a judgement.

**13 still cite their own rule and are left unmigrated at v0.22 spelling.**
Their rejection fires before the annotation can matter — every lexical case
(`FORM-4` comments, `FORM-2` tab indentation) and every case whose violation
is in a signature or effect row rather than a `let`.

**7 had been masked, all by [GRAM-4], and are restated.** Each carried
exactly one annotated `let` whose right-hand side held the violation under
test, so the restatement is the minimal one: delete the annotation, keep the
violation byte-for-byte. All 7 now reject for their own rule again, verified
by re-running the compiler.

Note which cases fell on which side: **every byte-level case is in the
untouched 13**, and none of the 7 tests a layout or lexical property. That
is why deleting an annotation from them perturbs nothing under test — the
ruling's caution about textual edits and byte-level properties applies to a
set that turned out to be disjoint from the set needing edits.

| case                                           | stage    | rule    | observed | disposition |
| form1-neg-unknown-construct                    | Parsing  | FORM-1  | FORM-1   | left at v0.22 |
| form3-neg-opname-bad-suffix                    | Parsing  | FORM-3  | FORM-3   | restated |
| form3-neg-region-param-missing-apostrophe      | Parsing  | FORM-3  | FORM-3   | left at v0.22 |
| form3-neg-requires-binding                     | Parsing  | FORM-3  | FORM-3   | left at v0.22 |
| form3-neg-reserved-mode-field                  | Parsing  | FORM-3  | FORM-3   | left at v0.22 |
| form3-neg-typeid-fn-name                       | Parsing  | FORM-3  | FORM-3   | left at v0.22 |
| form4-neg-comment                              | Lexing   | FORM-4  | FORM-4   | left at v0.22 |
| form5-neg-missing-suffix                       | Parsing  | FORM-5  | FORM-5   | restated |
| gram9-neg-constructor-in-call-argument         | Parsing  | GRAM-9  | GRAM-9   | left at v0.22 |
| gram9-neg-constructor-in-constructor-field     | Parsing  | GRAM-9  | GRAM-9   | restated |
| gram9-neg-nested-call                          | Parsing  | GRAM-9  | GRAM-9   | restated |
| x-eff-pure-combined-with-traps                 | Parsing  | EFF-1   | EFF-1    | left at v0.22 |
| x-eff-trailing-comma-row                       | Parsing  | EFF-1   | EFF-1    | left at v0.22 |
| x-eff-writes-missing-region                    | Parsing  | EFF-1   | EFF-1    | left at v0.22 |
| x-form-form2-tab-indent                        | Lexing   | FORM-2  | FORM-2   | left at v0.22 |
| x-form-form3-enum-name-ident                   | Parsing  | FORM-3  | FORM-3   | left at v0.22 |
| x-form-form4-block-comment                     | Lexing   | FORM-4  | FORM-4   | left at v0.22 |
| x-form-form5-op-arg-missing-suffix             | Parsing  | FORM-5  | FORM-5   | restated |
| x-gram-nested-op-in-construct-field            | Parsing  | GRAM-9  | GRAM-9   | restated |
| x-gram-nested-ucall-in-call-arg                | Parsing  | GRAM-9  | GRAM-9   | restated |

All 20 cite their recorded rule after the change, 20 of 20.

### A4 landed (`adc1a4d`) — the tool is complete, and the mandatory assertion found two more

A4 is the only class that reshapes a tree. On the token stream the reshape
is small for the reason the tool's shape exists: the arms' own braces become
the conditional's, so only the match's outer braces and the two arm headers
move and no indentation is computed anywhere. [ERR-2]'s asymmetry falls out
— an empty `False` arm becomes the else-free `if`, an empty `True` arm keeps
its block — and a value initializer keeps its `else` even when empty. A
`False`-first match is reported rather than guessed, since it would need its
bodies exchanged; the corpus writes `True` first in every measured case.

**Complete measured class figures**, over the 400 corpus files that parse:

| Class | Sites |
| --- | --- |
| A3 `let` annotations | 1992 |
| C1 respells | 886 |
| A1 argument lists | 691 |
| prelude constructors written | 99 (0 bare) |
| A4 conditionals | 260 (5 flattened) |
| files changed | 312 of 400 |

**The mandatory zero-surviving-Bool-match assertion earned its keep.** Run
over the migrated output it found **2** survivors, both correctly declined
by the tool because neither is a well-formed two-armed Bool match:

- `type5-neg-match-non-enum.wf` — a scalar scrutinee, so GRAM-6 never
  applies and TYPE-5 still fires. Cites TYPE-5 as written. **Leave.**
- `reject-err2-nonexhaustive.wf` — recorded ERR-2, **now cites OP-1**. This
  one needs a ruling and it is not the citation-masking question the 20
  were: its scrutinee is Bool, and under GRAM-6 a Bool match is rejected
  before exhaustiveness is ever consulted, so **ERR-2's concern cannot be
  tested through a Bool match at all under v0.23**. Restating it means
  giving it a source enum scrutinee — a change of witness, not of spelling.

Both sit outside the 20 because both *parse*; they fail semantically, so no
parse-based sweep would have surfaced them. Only the assertion did.

### M3b's two structural assertions — where they must live (measured, not guessed)

The 2026-08-08 instruction is to assert, on the migrated **trees** rather
than the text, that no `else` block holds exactly one `if_stmt` (the
unflattened form GRAM-6 forbids) and that no Bool-scrutinee `match`
survives. Both are right, and neither can live in the migration tool.

`FinalizedTopology` is `pub(crate)` and `FinalizedBundle::topology` is
`pub(crate)`, so **a bin cannot reach the tree at all** — a bin is a
separate crate and sees only `pub`. The same rules out
`compiler/tests/canonical_corpus.rs`, which is an integration test and
therefore also a separate crate. The assertions must be a **lib unit test**
under `compiler/src/`, which is better anyway: it makes them a standing
gate in `make check` rather than a one-shot check inside a tool nobody runs
again.

The text-level version was run and is recorded above; it found the two
survivors. The structural version subsumes it and is the one to land.

Note the assertions are not redundant with the checker even though GRAM-6
rejects both forms: a migrated file that fails earlier for another reason
never reaches the checker, so the rejection would not fire and the defect
would sit unnoticed in the corpus.

### Validation

`make -C compiler check` exit **2** and `make check` exit **2**, both at the
`test` step on the pre-existing 285, unchanged from the baseline at
`4da1717` — zero added and zero removed after M2, after the `box_new`
repair, and after infix resolution; the stage oracle reports zero changes
too. Lib goes 259 -> 279 passed across the three commits. `cargo clippy --all-targets` clean; `cargo fmt` applied. The 12
new tests are 11 in `semantic/tests/conditionals.rs` — the three rejections
pinned to their cited bytes, the enum scrutinee still taking `match`, the
flattened chain, the else-free form, the empty then-block, a non-Bool
condition, and both GIVE-1 carve-outs — plus the executing backend test.

## Round 6 (exec-0038e, 2026-08-08) — M1's semantic path, and a blocker that stops it

Two commits. M1's TYPE-5/GIVE-1 half is complete and cost nothing; its OP-2
half is complete and is coupled to the fixture migration; its FN-4 item is
**not started**, because the blocker below sits in front of it.

**Two claims in the round-5 brief did not hold and were corrected before
work started.** Branch tip `f7c8b19` **does not exist** in this repository
(`git cat-file -t` fatal); the real tip of
`task/0036-floor5-grammar-and-migration` is `12d9eb2`, carrying exactly the
described renderer commit. And a worktree **did** hold that branch —
`/Users/bytedance/do_not_scan/wf0038-r5`, clean and at the tip. It was left
alone rather than removed, so this round branched
`task/0038-floor5-semantic-and-migration` from `12d9eb2` and worked in its
own worktree. **The branch already contained main's tip `fb80bb1` as the
merge base, so no rebase was owed** — verified, not assumed. The candidate
still hashes to `ab257aa65874c4e6de167189b97cf706b5ca0045ccab86fdb54da83e2ba613da`,
recomputed here with `shasum -a 256`.

### `9931e0f` — TYPE-5 and GIVE-1 derivation

The inversion the round-3 brief described, and it is as small as that brief
said. `check_let` stops reading the `Mode` and `Type` children and each arm
takes what its right-hand side produces; `check_propagate_let` derives the
binder from the operand's own Ok payload instead of checking against the
annotation; `GiveContext` carries a `Cell` holding the mode and type the
first delivering `give` produced, later `give`s must agree exactly, and an
empty delivery set rejects at the `let_stmt` node.

**One consequence no brief predicted: it empties the expectation channel.**
After the amendment the only reader of an expected type in the whole
expression checker was `check_construct`, which used it to name the prelude
generic nominal. Now that the variant constructors write their arguments,
nothing reads it, so `check_expression_with_expected` and
`check_consuming_expression_with_expected` are gone and the `expected`
parameter is off `check_expression_in_context` and `check_construct`. The
three call sites that passed one — `return`, `set`, `give` — all re-checked
the result immediately afterwards, so no judgment was lost. This is round
4's finding 2 arriving in the code: [TYPE-6] already forbade the channel.

The propagate pre-pass in `nominal_instances.rs` that interned a `Result`
instance from the let's written annotation is deleted with it: that instance
is the callee's and its signature already interned it.

**Eight regressions**, `semantic::tests::derivation`, all v0.23-spelled,
including `let absent = None<buffer<u8>>();` — the round-3 blocker's own
witness, which had no legal spelling before the amendment. Lib went
**259 -> 267 passed / 271 failed**, and the failure SET is byte-identical to
`/Users/bytedance/do_not_scan/wf0038-baseline-failures.txt`: zero added,
zero removed.

### `9eed20a` — OP-2 operand-derived row selection

The first operand's exact type is the selection; every later operand is held
to the row's argument type for it, so "both operands must have one identical
exact type" falls out and cites TYPE-5 at the second operand atom. A written
type argument now cites **OP-1** — the judgment inverted, since v0.22 cited
FN-2 for its *absence*.

The same inversion covers the whole deleted class, which is wider than the
integer rows: Bool operations, enum equality, the float rows, `len`,
`box_new`, and `buffer_new`. **`buffer_new(n, v)` is the one row that
selects from its *second* operand** — its first is the u64 element count —
which is worth knowing before touching it. `cvt`, `reinterpret`,
`array_new` and `arena_new` are the retained class and are untouched, and
`finf`/`fnan` are nullary, so they keep a written result type and are the
one float row that still reads one.

**A required check was nearly lost and is restored, not dropped.** [STOR-5]'s
box-content judgment rode on the written referent type. It now tests the
*derived* referent and cites the operand that supplied it. A directly
slice-typed operand is the only path a region can take into box content:
struct fields and enum payloads are held to STOR-5 at their own
declarations, and `CheckedFlatElement` cannot be a slice, so no array,
buffer, or nominal referent can smuggle one in. Six more regressions, two of
them pinned to the exact cited bytes rather than the rule alone, because
[OP-2] and [STOR-5] each name *which* operand they land on.

**This half is coupled to the fixture migration, and that is new
information.** The round-5 brief expected M1 to land without breaking the
compiler's inline fixtures because it does not touch the Bool matches. It
breaks **14** of them anyway. Every one fails with `Op1 / InvalidOperation`
on a v0.22-spelled source that still writes `len<u8>(...)`, `ieq<i32>(...)`
or `box_new<slice<'r, u8>>(...)` — the new written-argument rejection
meeting un-migrated fixtures, one cause and no other. So OP-2 is coupled to
the inline-fixture migration exactly as GRAM-6 is coupled to the 87
`True() =>` arms, and M1 and M2 are less separable than the batch plan
assumes. Diffed as a set, not counted: those 14 are the only additions and
there are zero removals.

Lib after both commits: **259 passed / 285 failed**.

### The blocker: `box_new` loses the supply for its box nominal

**This needs an owner/lead ruling, it is the same shape as round 3's
`None()` blocker, and its likely repair moves the candidate bytes and all
three digest pins.**

`CheckedNominalKind::Box` instances are interned by exactly one site,
`ensure_nominal_type_head` in `nominal_instances.rs`, and only from a
**written `box<T>` type**. Checking is `&self`, so `check_box_new` can only
look the instance up — it cannot intern one. In v0.22 the supply was the let
annotation, `let owner: own box<u64> = box_new<u64>(value);`. A3 deletes it
and [STOR-2] derives the referent from `v`, so a purely local box has no
`box<T>` spelling anywhere in the unit and the lookup fails.

Reproduction, with a control that distinguishes the claimed cause — the two
programs contain byte-identical `box_new(41_u64)` calls:

```
# A — box<u64> named nowhere
fn main() -> own unit allocates(heap) {
  let owner = box_new(41_u64);
  let loaded = deref(owner);
  return unit;
}
whitefootc: Semantics/Compiler: InvalidResolution

# B — the same call, plus a signature that names box<u64>
fn take(b: own box<u64>) -> own unit pure { return unit; }
fn main() -> own unit allocates(heap) {
  let owner = box_new(41_u64);
  take(b: move owner);
  return unit;
}
(exit 0, no output)
```

A is a **compiler failure**, not a source rejection, which is the shape the
workflow says must never be reported as invalid source.

**Measured scope, on the 420-file basis** (string literals blanked, PCRE
lookbehind, cross-checked with Python `re`): **4** corpus files call
`box_new`; of those, **2** would be left with no surviving `box<T>` spelling
because their only one is the annotation A3 deletes —
`tests/conformance/cases/stor2-pos-box-new.wf` and
`tests/conformance/cases/stor3-pos-box-drop-region.wf`. The count is small;
the rule is not, because after A3 `let b = box_new(v);` has no legal
spelling in a unit that does not otherwise mention `box<T>`.

**Why an executor cannot settle it.** The repairs are a language or
architecture choice:

- (a) `box_new` joins [TYPE-5]'s retained-argument class, as the prelude
  variant constructors just did, on the same stated ground — spelling
  becomes `box_new<u64>(value)`. This **moves the delta**, so it re-keys
  `ab257aa6…` and all three pins, and it is in tension with [STOR-2]'s
  amended "`box_new(v)` returns `own box<T>` for `v`'s exact type T", which
  would also have to move.
- (b) Intern box nominals lazily during checking. This is the only repair
  that keeps the spelling the candidate already fixed, and it is a real
  change to the checker's `&self` discipline, not a local fix.
- (c) A pre-pass that derives `box_new` operand types before checking. It
  works for a literal operand and not in general, because the operand's type
  can come from a binding whose own type is derived. It does not close the
  gap.

**Neither (a) nor (b) is an executor's call**, and unlike round 3 there is a
second candidate repair that does *not* move normative bytes, so the choice
is genuinely open.

**What the ruling does not gate.** Both landed commits stand on their own and
neither depends on the answer: `box_new`'s lookup failure is reachable today
only from sources that are already un-migrated, and the 14 red fixtures are
the migration's, not the blocker's.

### Why round 6 stops here

FN-4's re-keyed premise is the only M1 item left, and it sits behind this
blocker in the same file. Opening M2 or M3 instead would mean starting the
atomic batch while a language question about one of its five transform
classes is open — the shape rounds 3 and 5 both refused. The honest boundary
is here, with the semantic path's two large halves landed, verified, and
regression-pinned, and one reproduced question in front of the rest.

## Successor brief (round 6)

Rounds 1–6 are discharged; do not re-derive them, re-assemble the candidate,
re-measure the nine figures, rebuild the renderer, or redo the TYPE-5,
GIVE-1 or OP-2 derivations. Base is
`task/0038-floor5-semantic-and-migration` at `9eed20a`.

1. **Get the `box_new` blocker ruled** before touching M1's remainder.
2. **FN-4's re-keyed premise** at `calls.rs` and `catalog.rs` — the only M1
   item left.
3. **M2 and M3 are more coupled than the batch plan assumes.** The 14 red
   fixtures are already OP-2's, and GRAM-6 adds the 87 `True() =>` arms on
   top, so the compiler's inline fixtures want migrating as one unit rather
   than twice. The migration tool should cover `compiler/src` fixtures and
   the 420-file corpus with the same transforms.
4. Then the conformance cases and the review packet, unchanged.

## Successor brief (round 5)

Rounds 1–4 and the renderer are discharged; do not re-derive them, and in
particular do not re-assemble the candidate, re-measure the nine figures, or
rebuild the renderer. Base is `task/0036-floor5-grammar-and-migration` rebased
onto main `fb80bb1`.

1. **The migration tool**, a bin under `compiler/src/bin/` (O1 binding
   condition 4: it ships re-runnable, because the `<`/`>` version re-runs it).
   Shape: lex the v0.22 source with the compiler's own lexer, rewrite the
   token stream, emit it with any spacing at all, then parse and
   `render_canonical`. Five classes: A1's deleted-class type arguments (1588),
   A3's let annotations (2003), A4's Bool matches to `if`/`else` with
   mandatory else-if flattening (262), C1's infix respells (378 + 519, with
   207 `ilt`/`igt` losing only their arguments), and the prelude-construction
   class (103, of which 101 rewrite and 2 stay bare). A3 and the prelude class
   are coupled: the written type argument at the `let` RHS site comes from the
   annotation A3 deletes, so capture it before dropping it.
2. **MANDATORY** (2026-08-08 ruling): assert zero surviving Bool-scrutinee
   matches rather than trusting the parse — a missed one is a GRAM-6
   rejection, i.e. a silent verdict change. Assert the 103/101/2 prelude split
   the same way.
3. **The semantic path**, unchanged from the round-4 brief.
4. The four repurposed and four new conformance cases, then the review packet.

The corpus gate is the migration's oracle and needs no separate evidence: when
`compiler/tests/canonical_corpus.rs` reports 418 round-tripped, 2 deliberately
non-canonical and 0 underived, every corpus file parses under v0.23 and is
canonical by construction.

## Successor brief (round 4)

Round 4 and everything above it is discharged; do not re-derive it, and in
particular do not re-assemble the candidate or re-measure the nine figures.
The base is branch `task/0036-floor5-grammar-and-migration`, rebased onto main
`879c503`.

1. **Build the canonical renderer** on `build_gap_styles`, per the finding
   above. Pin it with a round trip: the basis files are canonical under
   v0.22's rules apart from the fixtures that are deliberately not
   (`form2-neg-noncanonical-ws` is the one this round measured — enumerate
   that class from the manifest rather than assuming it is a singleton), so
   rendering a *parsed* file must reproduce its own bytes exactly for every
   file outside it. That is a ~420-file byte-exact control available before
   any migration happens, and it is the cheapest strong evidence on this
   branch.
2. **The scripted migration** (O1 binding condition 4 — it must ship
   re-runnable, not live in scratch). Five transform classes now: A1's
   deleted-class type arguments (1588), A3's let annotations (2003), A4's
   Bool matches to `if`/`else` with mandatory else-if flattening (262), C1's
   infix respells (378 + 519, with 207 `ilt`/`igt` losing only their
   arguments), and the new prelude-construction class (103, of which 101
   rewrite and 2 stay bare). Note it has no home yet: there is no `tools/`
   directory and a new top-level entry needs owner approval, so ask before
   inventing one — `compiler/src/bin/` is the precedent the grammar-tables
   generator set.
3. **The semantic path**, unchanged from the round-3 brief except that
   `check_construct` now also reads written `targs` for prelude ordinals
   5/6/11/13 and routes them through the same `generic_substitution` the
   source nominals use. That is the whole compiler side of the amendment,
   and it is a re-use rather than a new path.
4. **MANDATORY, from the 2026-08-08 ruling**: the migration must ASSERT zero
   surviving Bool-scrutinee matches rather than trust the parse — a missed
   one becomes a GRAM-6 rejection, i.e. a silent verdict change. Assert the
   103/101/2 prelude split the same way, for the same reason.
5. The four repurposed and four new conformance cases, then the review packet.

## Successor brief (round 3)

Rounds 1–5 of 0036 and this round are discharged; do not re-derive them.
The front end is done. What remains, in order:

1. ~~FORM-2 for `if_stmt`/`value_if`~~ — **DONE in round 2 (`5895526`)**. Retained below because the analysis explains the shape: it
   is bigger than the round-2 brief implies.** `is_block_bearing` needs
   the two productions, but the model underneath cannot hold them:
   `NodeRecord` carries a single `body_open`/`body_close` pair and
   `finalize/engine.rs:479` assigns it from the *last* `{` and `}` the
   node owns, so an `if_stmt` owning a then-block and an else-block keeps
   only the else-block's braces. Two brace pairs per node is a real
   change to `topology.rs`, `engine.rs`, `canonical.rs`'s `inside_body`
   at line 575, and `format.rs`. The `} else {` join line then follows the
   `RequiresBlock` precedent at `format.rs:165`, which is already the one
   production exempted from the break-after-close rule. Do this first:
   the corpus migration's FORM-2 canonical audit cannot run without it.
2. **The semantic path.** One measured finding sharpens the TYPE-5 half
   and was not in any earlier brief: **the checker already computes what
   the annotation declared.** `TypedExpression` (`semantic/check.rs:178`)
   carries both `mode: CheckedMode` and its expression's type, and
   `check_let` (`check/control.rs:443`) currently reads `Production::Mode`
   and `Production::Type`, calls `check_expression_with_expected(...,
   Some(expected))`, and then *rejects* on
   `value.expression.ty() != expected`. A3 does not ask for a new
   derivation engine; it asks that this site **take** `value.mode` and
   `value.expression.ty()` instead of comparing against written children
   that no longer parse. That is TYPE-5's "unique reconstruction, not
   inference" already present in the implementation.

   The consequent work is the plumbing around it, not the derivation:
   `check_let` must check the expression *before* it can name the
   binding's type, so the ordinary, propagate, `value_match` and
   `value_if` arms each need reordering; every `check_match(..,
   Some(expected))` call inverts to deriving from the delivery set; and
   the OWN-5 slice guard and `borrow_for_destination` re-key onto the
   derived mode. The rest is unchanged from the card: TYPE-5 derivation,
   [OP-2] operand-derived row selection reported at the second operand
   atom, [GIVE-1]'s contract inversion in `check/control.rs`'s `check_let`
   and `matches.rs`, `if_stmt`/`value_if` into the existing checked
   Bool-match, GRAM-6's three rejections, and FN-4's re-keyed premise.
   0036's site map is still accurate.
3. **The corpus migration** on the figures above, plus the compiler's own
   inline fixtures — note the 271 lib failures are Rust test sources under
   `compiler/src`, a body separate from the 420 `.wf` files.
4. The four new conformance cases, then the review packet.

**One caution earned this round.** Respelling the catalog in isolation
converted nine passing tests into failing ones, because tests that build
source *from* the catalog now emit `+wrap(a, b)` as a callee and cases
that resolved `iadd.wrap` by name no longer do. That is correct and
expected — infix resolution ([OP-1] (ii)) is what clears them — but it
means the catalog and the infix resolution path want to land together, and
a bare failure count will look worse before it looks better.

## Round 10 (exec-0038k, 2026-08-08) — the requires-block `let`, and a second defect in the same pass

Three commits. The adapter's failures fall 28 → 16 with **no case newly
failing**. Round 9's blocker is closed. One finding is reported rather than
absorbed: the authorized change removes a rejection the lead ruling did not
know was riding on the deleted annotation.

### The four controls, re-run rather than inherited

Sources in `/Users/bytedance/do_not_scan/wf0038-r9-controls`, each a complete
program, driven by `whitefootc --emit-llvm -o /dev/null <file>; echo "exit=$?"`.
Round 9's claims reproduce exactly:

| control | before | after |
|---|---|---|
| `let ok = ilt(a, 8_u64);` | `Semantics/Compiler: InvalidCanonicalTree`, exit 1 | exit 0 |
| `let ok: own Bool = ilt(a, 8_u64);` | `Parsing/Source [GRAM-4]`, exit 1 | `Parsing/Source [GRAM-4]`, exit 1 |
| `check a <= 8_u64 else trap …` | exit 0 | exit 0 |
| `check ilt(a, 8_u64) else trap …` | exit 0 | exit 0 |

The middle row is unchanged on purpose: A3 deleted the annotation from the
grammar, so the annotated spelling must stay a parse error.

### The failing set was 13, with one wrong count and one wrong name

`grep "^  Fail " | grep -c InvalidCanonicalTree` over the adapter's own output
gives 13, and the names are right everywhere except two places, which are worth
separating because they belong to different artifacts:

- **The count.** Both round 9's list above and round 10's brief say "the seven
  `fn8-*-requires-*`". There are **eight** — five `neg` and three `pos` — plus
  `fn8-trap-requires-false`, which the same glob also matches.
- **The name.** `fn8-neg-requires-member` does not exist: no case file, no
  manifest id. The member case is **`fn3-neg-requires-member`**, expecting
  `FN-3`. This name is correct in round 9's list and correct in round 10's
  brief; it was wrong only in the task-tracker card that opened round 10, so it
  is not a defect of either written record.

### `8838150` — the authorized change

`validate_requires_let` no longer reads a `Mode` or a `Type` child. Both
grounds hold in the v0.23 candidate at line 498:

```
$ grep -n "each let introduces a fresh clause-local" \
    governance/spec-evolution/kernel-spec-v0.23-candidate.md
498:… Its scope initially contains only the function parameters; each let
    introduces a fresh clause-local own copy value visible to later clause
    statements, and clause locals are not visible in the body. …
```

The mode needs no derivation because that sentence fixes it and no other mode
is spellable. The type needs none *for shape* because [TYPE-5] derives it in
`check_statement`, which runs on the very next line of `check_requires`.

**A correction to the brief's framing.** The early FN-8 structural pass the
spec describes is not this function — it is `check_requires_blocks` in
`compiler/src/resolution/engine/admission.rs`, which runs during resolution and
already reads neither annotation. `requires.rs::validate_requires_*` is a
later, semantic-subset pass. This matters because it is why the two rejected
approaches were never the only options, and why "stop requiring the children"
does not touch shape enforcement at all.

### `8ccd4d8` — a second defect the brief did not anticipate

With the annotation no longer demanded, the migrated corpus still failed: the
subset pass admitted only `Production::Call`, and v0.23's [FN-8] admits "an ANF
[GRAM-9] call to, **or infix spelling of**, a non-trapping, total
operation-table row". So `let permitted = x >= 0_i32;` — the exact form the
corpus now uses — was rejected `FN-8/InvalidRequires`.

The worse half was in the check position, which appeared to work. An `expr` is
`atom infix_tail?`, so `validate_requires_condition` found the expression's own
`atom`, validated it, and returned — never reading the operator row or the
right operand. Measured, not argued:

```
$ git checkout 8838150 -- compiler/src/semantic/check/requires.rs   # then rebuild
$ whitefootc --emit-llvm -o /dev/null p6-infix-right-subscript.wf; echo "exit=$?"
exit=0
$ git checkout HEAD -- compiler/src/semantic/check/requires.rs      # then rebuild
$ whitefootc --emit-llvm -o /dev/null p6-infix-right-subscript.wf; echo "exit=$?"
whitefootc: Semantics/Source [FN-8]: … kind: InvalidRequires
exit=1
```

That source is `check a <= xs[1_u64] else trap …` — a subscript, which [FN-8]
rejects by name. Admission asks the same `CheckedIntegerOperation::traps`
(`compiler/src/semantic/model.rs:412`) the ordinary checker asks, because the
bare `+ - * / %` forms carry the trapping mode with **no `.trap` suffix** and
the existing spelling filter cannot see them. Sharing that predicate avoids a
second reading of the operator table.

**Corrected 2026-08-08, same round, before review.** `8ccd4d8`'s commit message
overstated this, and the message stays as landed — the correction belongs here.
It claims a trapping operator, a `move`, a borrow, *and* a subscript escaped the
check position. **Only the subscript did**, and only that claim carried a
rebuild-on-both-sides reproduction. Measured at `efb5242`:
`check x + 1_i32 else trap …` gives `Semantics/Source [OP-5]
InvalidCheckCondition`, so a trapping operator **cannot** reach a check
condition at all — the condition must be Bool and trapping arithmetic yields an
integer. The `move` probe rejected `OWN-1 MoveOfCopy` for being a copy type,
which proves nothing about [FN-8], and the borrow probe never parsed: both are
**not measured**. One pre-existing escape existed, not four.

**But `traps()` is load-bearing, not prospective — a second correction, in the
other direction.** An earlier draft of this section called it speculative. That
was wrong, and measuring it settles it. The same commit newly *admits* the infix
spelling in a clause `let`, and the pre-existing filter recognizes only a
`.trap` suffix plus `{buffer_new, box_new, arena_new}` — bare `+ - * / %` carry
the trapping mode with no suffix and are not even reached by that filter on the
infix path. Removing the guard from the shipped code and rebuilding:

```
$ whitefootc --emit-llvm -o /dev/null p2-infix-trap-let.wf; echo "exit=$?"
exit=0            # guard removed: `let incremented = x + 1_i32;` compiles
whitefootc: Semantics/Source [FN-8]: … kind: InvalidRequires
exit=1            # guard restored
```

So admitting infix without sharing `CheckedIntegerOperation::traps`
(`compiler/src/semantic/model.rs:412`) would have opened a **new** hole in the
same commit, since [FN-8] admits only a "non-trapping, total" row. `8ccd4d8`
therefore did three things, each necessary: admitted a spelling [FN-8] requires
admitting, kept a trapping row from riding in on that admission, and closed one
pre-existing escape.

### What the two passes still reject

An earlier draft of this section said "probed, not inferred" over a list that was
part probed and part read off the code. Both are legitimate grounds for a
different kind of claim, so they are separated here rather than blended.

**Resolution's shape pass** (`check_requires_blocks`), read off the code, two
rows confirmed by probe:

| rejected | ground |
|---|---|
| a `check` in any non-final position — so a repeated `check`, or a `let` after one | probed: `p4`, `p5` → `Resolution/Source [FN-8] RequiresShape(InvalidEntry)` |
| an empty or all-`let` block, reported on the `requires_block` node | code: `RequiresShape(MissingFinalCheck)` |
| any entry that is neither an ordinary `let` nor a `check` — `doc`, `set`, `return`, and a `let` whose right-hand side is `propagate` or `value_match`, which `requires_entry_kind` classifies as `Other` for want of an `OrdinaryLetRhs` child | code |

**The semantic subset pass** (`validate_requires_*`), same split:

| rejected | ground |
|---|---|
| a trapping infix operator | probed: `p2` → `FN-8/InvalidRequires`, and see the guard-removal measurement above |
| a subscripted place in either infix operand | probed: `p6`, exit 0 before `8ccd4d8` and rejecting after |
| a bare-atom initializer — an initializer is a computation, and an atom is not one | probed: `p3`, `let candidate = x;` → `FN-8/InvalidRequires` |
| a clause local whose derived type is not a copy type | probed in round 11, nine rows, both sides |
| a callee resolving to a source function | code: the `DeclarationClass::Function` arm |
| a `*.trap` row, or `buffer_new`/`box_new`/`arena_new` | code: the spelling filter |
| a `move` or `BorrowExpr` operand | code: `has_fixed(Move)` and the `BorrowExpr` child test — **not** probed, and the two attempts to probe it in round 10 both failed to isolate it |

### Negative cases: none flipped

Every negative in the family still rejects. Five left the failure set by
reaching their recorded rule and kind — `fn3-neg-requires-member` (FN-3),
`fn8-neg-requires-eeq-integer` (OP-1), `-missing-traps` (EFF-2),
`-trapping-op` (FN-8), `-user-call` (FN-8) — and the seven already passing
(`-doc-only-clause`, `-control`, `-local-in-body`, `-no-check`,
`-non-bool-check`, `-set`, `form3-neg-requires-binding`) are absent from the
after set too. The set diff arrived empty in both directions of the gate and
the adapter.

`fn8-neg-requires-eeq-payload-enum` is the one of the 13 that did not clear.
It no longer fails internally — it now **rejects**, citing `OWN-1
BareAffineUse` where the manifest wants `OP-1`. That is not this pass: its
sibling `op1-neg-eeq-payload-enum`, which has no requires block at all, reaches
the identical `Reject(Some("OWN-1"))` and was already failing before this
round. Same pre-existing OP-1/OWN-1 precedence defect, outside this scope.

### Measured before/after

```
$ make -C compiler check; echo "exit=$?"
before  exit=2   test result: FAILED. 308 passed; 262 failed
after   exit=2   test result: FAILED. 318 passed; 253 failed
```

Nine tests newly pass (three `backend::tests::requires::*`, four
`semantic::tests::requires::*`, `base64::compiler_independent_base64_rfc_vectors_execute`,
`contracts::protected_fn3_rejections_keep_their_rule`), one is new here, and the
failure-set diff shows **nothing newly failing**. The gate is red before and
after for the pre-existing reason: the lib failures are unmigrated inline Rust
fixtures under `compiler/src`, which is task M3c, not this task.

```
$ cd compiler && cargo test --test conformance --locked --offline -- --ignored --nocapture
before  Pass=359  Fail=28  Skip=14
after   Pass=371  Fail=16  Skip=14
```

`make check` exits 2 on the same compiler target; every other step passes,
including `coverage (kernel-spec-v0.23-candidate.md): 128/128 rules covered`.

### Finding — the copy-value restriction now has no enforcement

Reported, not worked around. Before A3, `validate_requires_let` read the
written type and rejected a clause `let` whose type was not a copy type:

```rust
if !self.is_copy_type(self.parse_type(ty)?)? { return self.invalid_requires(entry); }
```

Removing the `Type` child removes that rejection, and nothing replaced it —
yet [FN-8] still says the clause local is an "own **copy** value". The
restriction is reachable, so this is a live spec/compiler discrepancy rather
than a dead branch. Two reproductions, both exit 0 at `7e80d92`:

```
$ whitefootc --emit-llvm -o /dev/null p1-noncopy-array.wf; echo "exit=$?"   # let xs = array_new<i32, 4>(0_i32);
exit=0
$ whitefootc --emit-llvm -o /dev/null p7-checked-result.wf; echo "exit=$?"  # let raised = x +checked 1_i32;
exit=0
```

`array<i32,4>` is non-copy by `is_copy_type`
(`compiler/src/semantic/check/nominals.rs:90`), and `Result<i32, Overflow>` is
non-copy because `CheckedNominal::is_copy` (`model.rs:288`) holds only for
enums whose every variant is fieldless. The reachable rows are `array_new` and
the `checked` arithmetic and partial-`cvt` families; `slice_of` is not one —
it needs a borrow operand, which `validate_requires_atom` rejects, and a region
that a clause has no way to bring into scope (`p8` → `OWN-3 UnresolvedUse`).

No conformance case covers a non-copy clause local, so nothing flipped and no
protected material weakened — which is exactly why this needs saying out loud
rather than showing up as a number.

**Why this was not fixed here.** The ruling's premise was that the pass "does
not need a type", and both listed alternatives were rejected. Neither rejection
actually blocks the fix: `check_statement` already derives and installs the
binding's type on the next line, so the restriction can be asserted on the type
ordinary checking produced — no reordering of the shape pass, no second
derivation. That is a one-place change, but where the assertion sits and which
diagnostic wins are the lead's call, and the ruling as written does not
authorize it. Recommend it as the next slice.








## Checks worth reaching for by default (exec-0038n, 2026-08-08)

Written at the lead's request for the next executor. None of these came from a
brief; each came from a slice in rounds 12–21 where the obvious check would
have passed and been wrong. They are habits, not process — four questions worth
asking before running something, and the instance that produced each.

**1. Prefer the observation that separates two hypotheses over the one
consistent with the hypothesis you hold.** Gathering evidence that fits your
current belief feels like verification and is not, because the same observation
usually fits the rival. Before running a check, ask what result would make you
believe the other thing; if no result would, the check is decorative.

- `own5-neg-slice-value-match` looked like a missing capability. Holding
  everything fixed but the join — two branches moving the *same* binding versus
  *different* bindings — separated "the rule is unimplemented" from "the rule is
  implemented and something runs first". The second was true.
- A mid-session `grep` returned pre-fix source and my commit ids were
  unreachable, which fits both "my work was reverted" and "a rebase is in
  flight". `git diff HEAD` being empty *combined with* HEAD containing the fix
  fits only the second. Either alone would have been consistent with the bad
  reading, and acting on it would have destroyed real work.

**2. Run a transform against the input it should have handled, never the output
it produced.** Testing a migrator, renderer, or formatter on its own output is a
fixed point: it always passes and proves nothing.

- The `give`/`propagate` constructor gap was only diagnosable from the
  *pre-migration* bytes, because the tool's correct behaviour is defined against
  what it should have seen. Re-running it on the file it had already broken
  changed nothing, which looked like correctness.

**3. Probe a check in every direction it can fail, not once.** Different
failure modes are different hypotheses about what the check is for, so one probe
demonstrates one of them.

- The rank check has two modes: a wrong rank, and a variant missing from the
  set. Breaking the value produced a test failure; deleting the variant produced
  a compile error. Proving only the first would have left the second — the
  actual defect being fixed — undemonstrated.

**4. When a migrated case behaves oddly, read the diff before reading the
compiler.** The program may simply have stopped being the program the case was
written about, in which case there is no compiler defect to find.

- `own3-pos-outlives-store`'s `Unsupported` looked like a capability gap and I
  located a real predicate defect behind it. The lead read the pre-migration
  bytes and found the annotation had named `'s` while the right-hand side named
  `'r` — the case's subject was deleted, and the predicate was not its cause.
  Correct diagnosis of the wrong question.

**Two more that generalize past their instances.**

**A mask's fix is itself a probe.** A masked failure means the number of hidden
problems is unknown, never one — so read the run immediately after removing a
mask carefully rather than treating it as confirmation. That is the moment
previously unreachable code first executes. Removing the OWN-5 mask surfaced a
stale expectation nobody was hunting; removing a dead driver-table entry
surfaced a second stale entry behind it.

**An operation against a stale baseline is invisible from inside the
operation, because the operation succeeds.** Commit before probing, resolve a
tip rather than relaying it, and do not write into a tree whose baseline someone
else is moving. A restore is only as precise as the commit it restores to.

**And the failure mode all of these share:** each one fails by looking like
success — a green case that tests nothing, a check that cannot fail, a
transform that agrees with itself, a stale baseline whose operation completes.
Anything whose failure mode is silence needs a probe designed to make it speak.

## Round 21 (exec-0038n, 2026-08-08) — the rank check made unskippable

One commit. Library **572 passed / 3 failed**, unchanged; coverage 128/128.

### Part 1: GRAM-6's rank was correct

Adding `Gram6` to the checked set **passes**. Its rank was right all along, so
the omission was benign for this instance — but the hole was real, and a check
that reports every rule verified while one is not is worse for that rule than
no check at all, because it converts an unknown into a false assurance.

### Part 2: the set is now walked from the enum

The checked set comes from `next_in_definition_order`, an exhaustive match,
rather than a hand-maintained array. There are now two exhaustive matches over
the rules — that one and `definition_rank` — and the test makes them **check
each other**: walking the chain must yield the ranks 0, 1, 2, … in order. A new
variant does not compile until it appears in both, and appearing in both is
exactly what being checked means.

`definition_rank` is unchanged and stays `pub const fn`, because
`borrows.rs` and `control/results.rs` use it in const assertions. The chain is
a second ordered source for that reason rather than a replacement.

**Measured both ways, because a check that cannot fail is the thing this slice
exists to remove:**

| deliberate break | result |
|---|---|
| `Sys2`'s rank changed 38 → 7 | test fails: "SYS-2 sits at chain position 38 but ranks 7" |
| `Sys2` dropped from the chain | **compile error**: `non-exhaustive patterns: SemanticRule::Sys2 not covered` |

The second is the guarantee in the compiler's own words.

`FIRST` and `next_in_definition_order` are `#[cfg(test)]`: their only purpose is
to make a check's set complete, and without the gate `-D warnings` fails the
production build on dead code. The compiler's demand therefore arrives when the
tests compile — inside `make check`, which is the gate that would catch a new
variant either way. Stated rather than left implicit, because it is the one
respect in which the guarantee is narrower than "any build".

### One process note, since it cost a cycle

Probing the check meant deliberately breaking the source and restoring it with
`git checkout --`, which **also reverted an uncommitted improvement** made
after the commit the probe restored to. The lint then failed for a reason
unrelated to the probe. The habit that avoids it: commit the improvement
*before* probing, not merely before the probe's revert — a restore is only as
precise as the commit it restores to.

## Round 20 (exec-0038n, 2026-08-08) — SYS-2 made representable, and what the coverage metric actually counts

One commit. Library **572 passed / 3 failed**, unchanged; adapter **386/2/13 →
387/2/13**; coverage **128/128, 0 uncovered**, negatives **44 → 45**.

### What the coverage metric counts — the measurement, not an argument

`tests/conformance/runner.py`:

```python
tagged |= set(c["rules"])          # every rule NAMED in any case's rules list
by_case  = tagged & rules
annotated = {a["rule"] for a in annots} & rules
covered  = by_case | annotated
```

**A rule counts as covered when any case merely names it in its `rules` list.**
Not when a case's expected verdict cites it, not when any case would fail if the
rule stopped being enforced, and with no notion of a rule's separate content
pieces. Measured against the corpus at this commit:

| | |
|---|---|
| rules in the specification | 128 |
| reported covered | **128** — 110 by case, 30 by annotation |
| with a positive exercise | 109 |
| with a negative citing them | **45** |
| **covered with no negative case at all** | **83** |
| covered with neither a positive nor a negative | **19** |

The runner already computes the positive and negative sets — it prints them as
`[+109/-45]` — but `covered` requires neither.

**The consequence, stated the way the packet needs it.** A positive case fails
if a rule wrongly *rejects*; a negative case fails if a rule stops *rejecting*.
So for the 83 rules with no negative, **a rule that silently stopped rejecting
would not be caught by this corpus**, and the figure "128/128 rules covered, 0
uncovered" does not distinguish that from a rule with a real negative.

**A fair qualification, because the number is not a defect count.** Many of the
83 have no negative *form* — GRAM-1 through GRAM-5 are grammar productions
exercised positively, the DIAG-* rules constrain diagnostic shape, EX-1 is a
worked example. "No negative" is not "untested" for those. The honest claim is
about what the *metric* asserts, not that 83 rules are unguarded: **the number
counts naming, and a reader is entitled to know that before treating it as
coverage.**

This is the second instance of the shape the wrong-kind FN-2 gap showed: the
metric is **per-rule, not per-content-piece**, so a rule with several distinct
violations counts as covered on the strength of any one of them.

### `5397bcb` — SYS-2 made representable

[DIAG-1]'s third clause could not be obeyed: `SemanticRule` had no `Sys2`
variant anywhere in `compiler/src`, so the correct citation was unrepresentable
rather than merely unused, and the system class cited TYPE-5.

**The rank is the part that could not be chosen freely.** `definition_rank` is
machine-checked against the active specification's definition order, so SYS-2's
rank is fixed by where the candidate defines it — line 769, between ERR-3 and
CLM-1, which is **rank 38**; CLM-1 and CLM-2 shift up by one. The check passing
is the evidence the rank is right rather than merely unique.

The citation then moves at the three argument-list sites in
`system_call_region_arguments` — absent, wrong count, wrong kind — exactly as the
callee-class fix moved the other two classes in round 15.

`sys2-neg-wrong-region-arg-count` gives SYS-2 its first negative, with a control:
two region arguments where `args_count` declares one rejects
`Semantics/Source [SYS-2]`; the same source with one exits 0.

### The [DIAG-1] rank insertion, measured rather than reasoned about

Inserting a rule into the definition rank is an observable behaviour surface,
not bookkeeping: [DIAG-1] settles simultaneous rejections by citing the
established rule whose definition appears first, so a new rank can change which
rule wins for a program violating two at once. Every one of the 422 corpus and
program sources was run through the pre-change binary and this one, comparing
exit code and cited rule:

```
exactly one verdict changed
  sys2-neg-wrong-region-arg-count   [TYPE-5]  ->  [SYS-2]
existing verdicts moved:            none
cases that compiled before and fail now: none
```

The one change is the new case itself, which is the intended effect.

**Why nothing else moved, offered as the mechanism rather than as luck.** The
simultaneity rule bites only where two rules are established at the same node.
The citation moved at three sites inside `system_call_region_arguments`, and
that function returns on the first rejection, so it cannot co-establish with
another rule at its node in the current implementation. That is consistent with
the measurement and with the code, but it is not an audit of every simultaneity
site — the differential is the evidence, and the mechanism is the explanation
for it.

**On where the rank lives, since it was asked explicitly: the enum's
declaration order is NOT load-bearing.** `definition_rank` is an explicit match
returning a literal per variant, and
`definition_rank_matches_the_active_specification` checks that those literals
sort in the same order as the rules' definition lines in the active
specification. The variant was placed next to its rank neighbours in the enum
for readability only; moving it elsewhere in the enum would change nothing.

### One thing found in passing, not fixed

`SemanticRule::Gram6` **has a `definition_rank` but is absent from the rank
test's `ALL` array**, so its rank is unverified against the specification while
every other variant's is checked. Forty variants have ranks; thirty-nine were
checked, now forty with SYS-2 added. GRAM-6 is the v0.23 conditional rule, so
this is most likely an omission from when it was introduced rather than a
decision. Reported rather than changed: adding it to the array is a one-line
change but it is a check gaining a subject, which is the lead's call.

### Validation

- `make check`: exit 2 at the compiler step, lib **572 passed / 3 failed** — the
  two activation-gated spec checks and the `RegionsAndBorrows` capability gap.
- Adapter **Pass=387 Fail=2 Skip=13**: `own3-pos-outlives-store` and
  `fn8-neg-requires-eeq-payload-enum`.
- `definition_rank_matches_the_active_specification` passes with 40 variants.
- `cargo clippy --all-targets -D warnings` exit 0; `cargo fmt --check` exit 0.
- Manifest: 402 case rows, all parse.
## Round 19 (exec-0038n, 2026-08-08) — the emptied-case sweep

Read-only over the corpus, pinned at `40fe986`. **One new emptied case beyond
the two known**, plus a third class the framing did not anticipate. Nothing was
restated; every disposition below is the lead's or the owner's.

### Method, and why the first attempt was thrown away

The tell is a case whose `doc` names a construct its source no longer contains,
cross-checked against the construct its manifest rule is about. A first pass
matched operation names as substrings and returned **97** candidates — `ile`
inside "wh**ile**" and "comp**ile**", `ine` inside "l**ine**" and "def**ine**".
That list was discarded rather than reported: a sweep whose hits are mostly
artefacts is worse than no sweep, because it buries the real ones. With
word-boundary matching the candidate set is **19**, and each was then read by
hand against its own pre-migration bytes.

### The result

| class | count | what it means |
|---|---|---|
| **emptied** — stated subject entirely gone, case green | **2** | `fn2-pos-explicit-instantiation` (known), **`type5-pos-explicit` (new)** |
| **subject shifted** — still earns its rule, but for a different violation than recorded | 3 | `op1-neg-eeq-integer`, `op1-neg-ene-integer`, `op1-neg-ineg-unsigned` |
| doc stale only, subject intact | 13 | the respelled arithmetic and comparison cases |
| false positive | 1 | `stor1-pos-frame-resident` |

Plus the one already retired in round 17, `fn2-neg-implicit-instantiation`,
which was the red half of the same class.

### The new hit: `type5-pos-explicit`

Its doc is its subject, and both halves of it are gone:

> "Every let states full mode+type; the call states its type arg; types match
> [TYPE-5]."

```
pre-migration                          now
  let a: own i32 = 41_i32;               let a = 41_i32;
  let b: own i32 = addk(x: a);           let b = addk(x: a);
  return iadd.trap<i32>(x, 1_i32);       return x + 1_i32;
```

**No `let` states a mode and type, and no call states a type argument** — the
two things it exists to demonstrate. It is a `run` case and it passes, so
nothing sees it. Same class as `fn2-pos-explicit-instantiation`, found the same
way: by reading, not by a gate.

### The third class the framing did not anticipate: subject shifted

Three OP-1 negatives were written about a **written type argument** and now
reject on an **operand domain** instead:

```
op1-neg-eeq-integer     eeq<u32>(left, right)   ->  eeq(left, right)
op1-neg-ene-integer     ene<u32>(left, right)   ->  ene(left, right)
op1-neg-ineg-unsigned   ineg.wrap<u8>(1_u8)     ->  ineg.wrap(1_u8)
```

Each doc says the operation "does not accept an integer type argument" or "has
exactly one signed integer type argument". A1 deleted the argument, so the
recorded violation no longer exists — but each still rejects OP-1, because the
*operands* are outside the row's domain.

**This class is harder to see than an emptied case.** An emptied case can at
least be caught by asking whether the rule still fires; these still fire, and
they cite the correct rule, so even a rule-level audit passes them. Only reading
the doc against the source finds them. They are not urgent — each still tests
something real that OP-1 owns — but the corpus records three concerns it no
longer covers, and the reviewer would have no way to know.

### What is stale but sound

Thirteen cases name a respelled operation in their prose while their source uses
the operator: `iadd.checked` against `+checked`, `ieq` against `==`, and so on.
The operation is still exercised; only the prose is out of date. **Four of them
— the `ieq`/`ine` cases — will have correct docs again once the infix comparison
reversal lands**, so they should not be touched now. The nine arithmetic ones
will not self-correct.

One candidate is a plain false positive and is named so it is not re-found:
`stor1-pos-frame-resident`'s doc says there is **no** per-binding storage
annotation, which is an assertion about absence, not a subject that died.

### For the review packet

The count is small, and the shape is what matters rather than the number.
Nothing in this repository looks for a case that passes without exercising its
own concern, and this sweep needed a human reading of 19 docs against 19
sources. So the corpus's green figure does not distinguish "passes because the
rule holds" from "passes because the case no longer tests what it says", and
after this sweep we can say the difference is **at least two cases, plus three
more that changed what they test**. A reviewer approving bytes is entitled to
that sentence.

The selection effect is worth stating in the packet too: the red half of this
class was caught one case at a time over four rounds because failure surfaced
it, and the green half needed a deliberate search. That asymmetry is a property
of the process, not of the migration.

### Scope and limits

- Pinned at `40fe986`; the corpus is being re-migrated concurrently by the infix
  reversal, and the four `ieq`/`ine` doc-stale hits are the ones that change.
- Conformance cases only (401). `tests/programs/` carries no manifest `doc`, so
  the tell does not exist there — those 20 files are **not covered** by this
  sweep and would need a different signal.
- The sweep detects a doc that names a *construct*. A case whose subject is
  stated only in prose too general to name one — "types match exactly" — is
  reachable only by reading, and `type5-pos-explicit` was found that way rather
  than by the mechanical signal alone.
## Round 18 (exec-0038n, 2026-08-08) — the give/propagate positions, OWN-5's ordering, and the A3 sweep

Three commits. The library gate goes **570 passed / 5 failed → 572 / 3** and the
adapter **384/4/13 → 386/2/13**. Nothing newly failing, and no acceptance lost
anywhere — measured rather than argued.

### `59ee50e` — the tool learns the two positions it never had

[TYPE-5] mandates written prelude-constructor arguments in **every** position;
the tool wrote them in two, an annotated `let` and a `return`. Constructors in a
`give` or a `propagate` were therefore left bare by the migration and are now
illegal. Fixing the two live sites without teaching the tool would leave a
re-run to re-break them, which is why both land together.

The two positions are not the same rule:

- A **`give`** inside a value initializer takes exactly what a directly assigned
  constructor takes — the binder's annotation is the delivered type. The direct
  rule missed it only because the constructor sits inside an arm rather than
  after the `=`.
- A **`propagate`** is the one position whose arguments come from two places.
  `let x: own T = propagate Err(error: e);` in a function returning
  `Result<_, E>` needs `Err<T, E>`: the Ok half from the annotation, the error
  half from the declared result. Neither source alone is enough, which is why
  `result_type` is now threaded into `annotated_let` beside its existing use.

Verified against the **pre-migration bytes** of both live sites, which is the
input the tool should have handled the first time, and both migrate to sources
that compile at exit 0. The live sites are then repaired with exactly those
bytes rather than hand-derived spellings.

### `9f9cbb5` — OWN-5 is judged before the join, and the rule keeps one home

[OWN-5]'s slice-valued-delivery prohibition was judged in `check_let`, one step
after the branch-state join that runs inside `check_match` and `check_if`.
`join_states` stops with `Unsupported { OwnershipJoin }` whenever two branches
leave their bindings in states differing by more than region claims — which is
exactly what a slice-valued join looks like when written on purpose — so a
capability stop stood in front of a source rejection.

| | before | after |
|---|---|---|
| both branches move the **same** binding | OWN-5 | OWN-5 |
| branches move **different** bindings | `Unsupported` | **OWN-5** |

The rejection moves to the delivery site and the later copy is **deleted rather
than duplicated**, so the rule is judged once.

**One consequence beyond the reported case**, stated rather than left to be
found: a slice delivery in borrow mode previously reached the RegionsAndBorrows
stop first and now reaches OWN-5, because the prohibition holds whatever the
mode. Same masking, same place.

**Acceptance is unchanged, and this was the ruling's condition.** Every one of
the 421 corpus and program sources was run through the parent binary and this
one, comparing exit code and cited rule:

```
exactly one verdict changed
  own5-neg-slice-value-match   Unsupported { OwnershipJoin }  ->  [OWN-5]
cases that compiled before and fail now: none
```

A stale expectation surfaced with the mask and is brought to the delta:
`slice_value_matches_…` asserted v0.22's "a match statement whose arms" where
v0.23 extends the prohibition to `value_if` and the fix names both forms. Rule
and kind unchanged; only the mechanical fix's prose follows, and that assertion
had never reached it because the join stopped first.

### The A3 counterexample sweep — the answer is one, and it is the known one

A3's premise is that a binder's mode and type are exactly what its right-hand
side produces. `own3-pos-outlives-store` is a counterexample: its annotation
named `'s` while its right-hand side named `'r`. The question is whether the
400-file migration ran over others.

**Regions**, the class that can differ silently, since Whitefoot has no coercion
and a type or mode mismatch was already a v0.22 rejection:

```
annotated `let` bindings scanned:       1954
  ...whose annotation names a region:     68
  ...naming a region the RHS does not:     1
      tests/conformance/cases/own3-pos-outlives-store.wf:6
      let q: &'s i32 = &'r a;
```

Fifty-seven further sites where the two region sets merely *differ* were
examined and are not counterexamples: in every one the annotation names no
region at all and the right-hand side's region is a **call's region argument**
whose result type is region-free — `let total: own u64 = args_count<'a>(args:
&'a args);`. The annotation is correct there and the deletion is sound.

**Mode and type**, the other half of the question, checked directly rather than
by argument. An annotation naming a mode or type its RHS did not produce could
only survive in a case *expected to reject*, since a positive case would have
been a v0.22 rejection itself. Every rejection-expecting case was run:

```
conformance cases expecting a rejection:      197
  ...that now compile clean (violation gone):   0
```

**So the sweep's answer is: exactly one counterexample corpus-wide, and it is
the one already found.** A3's premise holds everywhere else the migration
touched. The single site remains a real question for the candidate's §5 — v0.23
removes the ability to state a destination region for a local binding, and the
accepted-set account lists three deliberate narrowings without it — but it is a
question about one construct, not about 1954 deletions.

### What this sweep does *not* cover

It finds a case whose **violation** the migration deleted, because such a case
fails loudly. It cannot find a **positive** case whose subject the migration
deleted, because that case stays green — round 17's
`fn2-pos-explicit-instantiation` is one, found by reading rather than by any
gate. That class is still open and is the sweep worth running next.

### Validation

- `make check`: exit 2 at the compiler step, lib **572 passed / 3 failed** — the
  two activation-gated spec checks and the `RegionsAndBorrows` capability gap.
- Adapter **Pass=386 Fail=2 Skip=13**: `own3-pos-outlives-store` and
  `fn8-neg-requires-eeq-payload-enum`.
- Conformance coverage **128/128 rules, 0 uncovered**.
- Migration tool: **31 tests pass**.
- `cargo clippy --all-targets -D warnings` exit 0; `cargo fmt` applied.
## Round 17 (exec-0038n, 2026-08-08) — the `fn2` residue disposed, and two things the enumeration found

One commit. The library gate goes **569 passed / 6 failed → 570 / 5** and the
adapter holds at **384/4** with Skip 14 → 13. Nothing newly failing.

### The enumeration, which is the precondition and not a formality

FN-2's negative content and its live carriers, checked rather than assumed:

| piece | live carrier |
|---|---|
| missing instantiation argument on a user generic | `fn2-neg-eeq-implicit-type` (repurposed), rejects FN-2 |
| wrong region-argument count on a user function | `type5-neg-wrong-region-arg-count`, rejects FN-2 |
| region-bearing generic type argument | `fn2-neg-function-region-bearing-targ`, `fn2-neg-nominal-region-bearing-targ` |

`fn2-neg-implicit-instantiation` carries **none** of them. Its own content — "a
generic *op* used with no explicit instantiation argument" — is not FN-2's
content at all under v0.23, because a table operation carries no written
argument and [DIAG-1] gives it the rule [OP-2] selects. So the case is not
redundant but **dead**, and retiring it drops nothing. Both the case file and
its manifest row go: the manifest has no `retired` status, and every row pairs
with a file (402 rows → 401).

### The cascade, and the second witness earning its keep

`driver.rs`'s hard-coded negative table embedded the retired case, so that entry
goes with it — it names a case that no longer exists, rather than being an
assertion dropped on its own.

**With the dead entry gone, the driver test failed on the next one.**
`x-match-give1-wrong-type`: the table said TYPE-5, the manifest said GIVE-1, and
the compiler said GIVE-1. The M3b dispositions ruling (d) moved that citation
TYPE-5 → GIVE-1 with the source unchanged; the manifest was updated then and
this witness was not. **That desync is exactly what a duplicated expectation
exists to catch, and it had been sitting undetected behind the dead entry
failing in front of it.**

All 21 entries were then checked three ways — driver rule, manifest rule, and
the compiler's actual verdict — rather than only the one that happened to fail.
Exactly one disagreed. It is corrected by hand against the ruling and never
derived from the manifest, because deriving it would destroy what it is for.

### Two findings the enumeration surfaced, neither disposed here

**1. A piece of FN-2 negative content has no conformance coverage at all.** A
**wrong-kind** instantiation argument — a const where the parameter declares a
type, or the reverse — is FN-2's under [DIAG-1] ("a missing, wrong-kind,
wrong-count, or wrong-domain argument"), and is observed to reject:

```
fn marker<T>() -> own unit pure { return unit; }
  marker<4>();     ->  Semantics/Source [FN-2], kind TypeMismatch
```

The only thing covering it is the library test
`generic_argument_kinds_and_const_parameter_types_are_checked`. A lib test is
not conformance coverage, and the coverage gate counts FN-2 as covered on the
strength of the four carriers above. Candidate work for M4.

**2. `fn2-pos-explicit-instantiation` PASSES while testing nothing, and no gate
can see it.** Its whole subject is that explicit instantiation arguments are
written and monomorphized; A1 respelled them out of its source, which now reads

```
let a = 40_i32 + 2_i32;
let b = a *wrap 1_i32;
```

with **no instantiation argument anywhere**, while its doc still claims "Generic
ops are instantiated with explicit type arguments [FN-2]". It is the same class
as the case retired above — content the migration deleted — but **green**, so
the pinned failure set cannot surface it and neither can the adapter. This is
the hazard the batch has been guarding against all week, in its invisible form.

A restatement is verified and ready, preserving its `run` expectation and its
subject by moving it to a user generic, which still writes and monomorphizes
explicit arguments:

```
let a = Held(v: 42_i32);
let b = pick<Held>(value: move a);
check b.v == 42_i32 else trap "mono drift";
```

observed to exit 0. Not applied: it is a protected conformance case and the
disposition is the owner's, not this unit's.

**The general point is worth more than either case.** Both were found by asking
what a case's content *is* now, not by watching a gate. A migration that deletes
a construct silently empties every case whose subject was that construct, and
half of those cases go green rather than red. Nothing in the repository looks
for a case that passes without exercising its own concern.

### Validation

- `make check`: **exit 2** at the compiler step, lib **570 passed / 5 failed**;
  the driver test clears and nothing else moves.
- Conformance coverage **128/128 rules, 0 uncovered** — FN-2 keeps four live
  carriers after the retirement.
- Adapter **Pass=384 Fail=4 Skip=13**, the one fewer skip being the retired
  case. The four are `own3-pos-outlives-store`, `x-give-result-aggregate`,
  `fn8-neg-requires-eeq-payload-enum`, `own5-neg-slice-value-match`.
- `cargo fmt` applied; the manifest's six non-JSON lines remain pre-existing.
## Round 16 (exec-0038n, 2026-08-08) — the three remaining adapter failures, diagnosed

No compiler or corpus change: this round is diagnosis, as scoped. Each cause is
established by a control that separates it from the neighbouring hypothesis, and
**one of the three is classified differently from the task card** — with the
observation that settles it.

### 1. `x-give-result-aggregate` and `semantic::tests::result_construction_…` — a MIGRATION gap, not a compiler defect

Both reach TYPE-5 TypeMismatch, and the shared cause is not in the compiler.
[TYPE-5] mandates written arguments on the prelude generic constructors in
**every** position: "the prelude generic nominals `Option<T>` and `Result<T, E>`
through their variant constructors `None`, `Some`, `Ok`, and `Err` … their
absence … is a hard error citing TYPE-5 at the complete `construct`".

`whitefoot-migrate` writes those arguments in exactly two positions —
an annotated `let` (`write_constructor_arguments`) and a `return`
(`returned_constructor`). It has **no rule for `give` or for `propagate`**, so
constructors in those two positions were left bare by the migration and are now
illegal.

Measured, each with its written-argument control:

```
give Ok(value: 1_u64);                       TYPE-5
give Ok<u64, u64>(value: 1_u64);             exit 0
propagate Err(error: error);                 TYPE-5
propagate Err<i32, StepError>(error: error); exit 0
```

**Scope measured, not estimated: 3 sites in 2 files, tree-wide.**

```
git grep -n -E "(give|propagate) (Ok|Err|Some|None)\(" \
  -- tests/conformance/cases tests/programs compiler/src compiler/tests
```

returns `x-give-result-aggregate.wf:3`, `:5` and
`compiler/src/semantic/tests.rs:737`, and nothing else. The `return` hits the
same sweep reports are all Rust `return Err(BackendFailure::…)`, not Whitefoot.

The two halves share one root cause and one fix, so they should land together;
the conformance half is protected material and was not touched. Teaching the
tool the two missing positions would additionally make a re-run idempotent
rather than re-breaking them.

### 2. `own5-neg-slice-value-match` — a MASKED negative, not a missing capability

The task card classifies this as a capability gap. **It is an ordering problem,
and the OWN-5 rejection it wants is implemented, correct, and reachable.**

`check_let`'s slice-valued-delivery prohibition is at
`semantic/check/control.rs:523`, keyed on the *derived* delivery type exactly as
the candidate's OWN-5 requires. What runs first is `join_states`
(`semantic/check/control/matches.rs:658`), which refuses any branch join whose
bindings differ by more than region claims and stops with
`Unsupported { OwnershipJoin }`.

The discriminator holds everything fixed but the join:

```
give move left; / give move left;    both branches move the SAME binding
  -> Semantics/Source [OWN-5]                      the rejection fires

give move left; / give move right;   different bindings (the case)
  -> Semantics/Unsupported { OwnershipJoin }       the rejection is masked
```

So nothing is unimplemented here. A capability stop is standing in front of a
source rejection, which is the masking pattern this batch has now hit in four
places. **This is the more serious of the two gaps for exactly the reason the
card gives — a negative that never reaches its rejection tests nothing — but the
repair is small rather than capability work**: a slice-valued delivery is
prohibited outright, so OWN-5 can be judged before the join is attempted. That
is a compiler change and is not taken here.

### 3. `own3-pos-outlives-store` — a GENUINE capability gap, located to one predicate

Classified correctly by the card. The trigger is isolated, and the `deref` in
the case is irrelevant to it:

| source | outcome |
|---|---|
| one region, borrow that region, deref | **exit 0** |
| nested regions, borrow the **inner** region, deref | **exit 0** |
| nested regions, borrow the **outer** region, deref | `Unsupported { RegionsAndBorrows }` |
| nested regions, borrow the **outer** region, no deref | `Unsupported { RegionsAndBorrows }` |

The predicate is `borrow_holder_scope_supported`
(`semantic/check/borrows.rs:894`), whose whole body reduces to

```rust
holder_scope.parent() == Some(self.region_declaration(region)?.scope())
```

— the holder binding's scope must be a *direct child* of the borrowed region's
scope, i.e. **the borrow's region must be the immediately enclosing region**.
[OWN-3] permits any enclosing region that outlives, which is strictly wider, and
the case is named for precisely that shape.

Worth noting for whoever scopes the repair: **the general relation already
exists in the same file** — `region_outlives` at `borrows.rs:883`, used for
OWN-4's `InvalidBorrowLifetime`. So this is a predicate testing scope-parent
identity where the outlives relation is what OWN-3 means, with that relation
implemented next door. Whether widening it is sound depends on the loan
bookkeeping downstream, which is why it is reported as a scoping question rather
than attempted.

### What this means for activation

Of the four adapter failures, **one is not a compiler defect at all** — it is
unfinished migration in two source positions, and it takes the lib sibling with
it. Of the remaining three, one is a masked negative whose rule is already
implemented, one is a located capability gap, and `fn8-neg-requires-eeq-payload-enum`
is tracked separately. None is unexplained.
## Round 15 (exec-0038n, 2026-08-08) — citation by callee class, and the four rulings it forced

Two commits on `task/0023-citation-by-callee-class`, rebased onto `main` at
`dfec564`. The library gate goes **568 passed / 6 failed → 569 / 6** with the
same six names, and the adapter **383/5/14 → 384/4/14**. Nothing newly failing
against `main`'s pinned sets in either lane. Exit codes from `$?`.

### `d6e66b5` — the compiler cites by callee class

[DIAG-1] selects the cited rule by the callee's class; the compiler selected
from the *kind* of argument problem, so it was wrong in both directions at
once. That is why two units reported it as two blockers.

| shape | before | after |
|---|---|---|
| `pick(value: move a)` user-generic, missing | TYPE-5 | **FN-2** |
| `pick<Held, Held>(…)` user-generic, wrong count | TYPE-5 | **FN-2** |
| `pick<Held>(value: move a)` correct | exit 0 | exit 0 |
| `cvt(value)` table op, missing | FN-2 | **TYPE-5** |
| `cvt<i32>(value)` table op, wrong count | OP-1 | OP-1 |
| `array_new(0_u8)` table op, missing | FN-2 | **TYPE-5** |
| `finf()` table op, missing | TYPE-5 | TYPE-5 |
| `Pair(v: 1_i32)` construct, missing | TYPE-5 | TYPE-5 |

`finf`/`fnan` were already correct and `retained_operation_type_argument`'s doc
comment already carried the reason, so the two wrong table sites were brought
to their sibling's reading rather than to a new one.

**The user-generic half needed the rule threaded, not replaced.**
`generic_substitution` reads one argument list for two callee classes that
[DIAG-1] assigns different rules — a user-generic call cites FN-2, a generic
nominal's construct cites TYPE-5 "at the complete `construct`". The rule now
arrives from the caller that knows its own class instead of being chosen
inside from the shape of the failure: one parameter, two call sites, no new
machinery.

Two recorded expectations moved, both flagged in advance as witnesses to this
question rather than found convenient here — round 14 named the `cvt(value)`
one explicitly — and `generic_argument_kinds_…`'s two wrong-kind arguments on
a user-generic call, which recorded TYPE-5 where [DIAG-1] gives a wrong-kind
argument to the callee's class.

New test `the_cited_rule_follows_the_callee_class_and_not_the_argument_problem`
holds one argument problem fixed across the classes so only the callee varies,
with the generic-nominal construct as the control that the rule is not simply
keyed on the shared argument-list reader.

### The three verdicts the fix moved, and why the executor stopped

The adapter went 383/5/14 → **380/8/14**: `type5-neg-wrong-region-arg-count`,
`type5-neg-shared-for-uniq-arg` and `x-fn-own-arg-for-ref-param` all moved
TYPE-5 → FN-2. Conformance material, so the unit stopped and reported rather
than integrating a net −3 on its own judgement. All three call a function that
is region-parametric but not generic.

### `17f68ac` — the four rulings carried out

**Ruling 1, the citation.** `type5-neg-wrong-region-arg-count` moves TYPE-5 →
FN-2 in the manifest, source untouched. [TYPE-5]'s own sentence assigns "type,
region, and const arguments for user generics [FN-2]", so its recorded rule
was wrong independently of any compiler change — the same shape as
`own1-neg-match-move-through-borrow`'s OWN-1 → OWN-5. The id is kept: an id is
a stable identifier, not a claim.

**Ruling 2, the sources.** The other two omitted a mandatory region argument
and were rejected for that before reaching the mode mismatch that is their
subject. **The mask predates the fix**; the fix only made it visible, because
the masking citation happened to be TYPE-5 and matched the row by coincidence.
Each now writes its region argument — `x-fn-own-arg-for-ref-param` also gains
the `region 'r` block it needs in order to have one to write — and both return
to `Semantics/Source [TYPE-5]`, observed rather than assumed.

**Ruling 3, the retirement — and its precondition changed the outcome.** The
required enumeration of FN-2's negative content against live cases:

| piece | live carrier |
|---|---|
| region-bearing generic argument at the `targ` | `fn2-neg-function-region-bearing-targ`, `fn2-neg-nominal-region-bearing-targ` — both runnable, both reject FN-2 |
| wrong region-argument count on a user function | `type5-neg-wrong-region-arg-count`, after ruling 1 |
| **missing instantiation argument on a user generic** | **none** — `fn2-neg-implicit-instantiation` is the only carrier and is pending on a source A1 makes legal |

So retiring would have left the third piece uncovered, and the ruling's
condition applies: `fn2-neg-eeq-implicit-type` is **repurposed onto it rather
than retired**. Its old concern is gone twice over — A1 deletes `eeq`'s written
argument, and [DIAG-1] gives a table operation the rule [OP-2] selects rather
than FN-2 in any event. It now states a user-generic call with no instantiation
argument and rejects FN-2. The enumeration is the part that earned its keep:
retiring on the first reading would have silently dropped coverage.

**Ruling 4, the stale reason.** `fn2-neg-implicit-instantiation` keeps
`pending` and its reason is **corrected rather than deleted**. The old reason —
"the active compiler does not yet implement the complete generic-instantiation
judgment and its FN-2 diagnostic path" — is stale in both halves: that path
exists as of `d6e66b5`, and it is not what blocks the case. Its source reads
`let a = 40_i32 + 2_i32;`, legal v0.23, accepted at exit 0.

### A correction to this record's own pinned failure set

`6ad23a1` attributes `driver::…::compiler_independent_negative_cases_…` to the
citation defect. **It is not, and the fix does not clear it** — that test
demands FN-2 from `fn2-neg-implicit-instantiation.wf`, measured exit 0. Its
concern died with A1's bytes, which is round 8's finding-2 class. The
attribution matters because the pinned table is the reference for telling a
real regression from an expected one, and an entry pointing at a cause that
cannot produce it sends the next reader hunting a fixed defect.

### Not fixed, and reported rather than forced

[DIAG-1]'s third clause: a system operation's region arguments must cite SYS-2,
and `system.rs` cites TYPE-5. **`SemanticRule` has no `Sys2` variant anywhere
in `compiler/src`**, so the correct citation is unrepresentable rather than
merely unused, and no conformance case expects a SYS-2 rejection — SYS-2
appears only in an accept row's rule list. Registered as its own slice.

### Validation

- `make -C compiler check`: **exit 2**, lib **569 passed / 6 failed**, the same
  six names as `main`'s pinned set.
- Adapter **Pass=384 Fail=4 Skip=14**: `main`'s pinned five less the repurposed
  case, which now passes. Nothing newly failing.
- `make check`: exit 2 at the same compiler step; conformance coverage
  **128/128 rules, 0 uncovered**.
- The manifest's six non-JSON lines are pre-existing and identical on `main`;
  the three edited rows parse.
- `cargo clippy --all-targets -D warnings` exit 0; `cargo fmt --check` exit 0.
## Round 14 (exec-0038n, 2026-08-08) — the rebase, `slice_of`, and the six ruled restatements

Five commits on top of round 13, on `task/0038-m3c-inline-fixtures` rebased
onto the trunk. The library gate goes **537 passed / 36 failed → 568 / 7** and
the adapter **381/7/14 → 383/5/14**, with **nothing newly failing at any
step**. Exit codes read from `$?`, never through a pipe.

### The rebase, and the one prediction it falsified

`8dc6a50` was verified reachable from the trunk and **not** reachable from
round 13's base before it was trusted. Two conflicts, both resolved by keeping
both sides: `migrate/main.rs` (the conformance unit's `mod manifest` beside
this unit's `mod embedded`) and the record's status block.

Post-rebase, before any work: lib **537 / 36**, set diff **3 cleared, 0
arrived**. The prediction was that all fourteen branch-scope tests would
clear. **Ten did not** — and they had not stayed put, they *moved*:

```
Resolution [TYPE-6] DeclarationCollision  ->  Semantics [FN-2] InvalidOperation
at tests/programs/wfgrep.wf:196
  let view = slice_of(&'report_prefix error_prefix);
```

`wfgrep.wf` uses `slice_of` twelve times, so those ten were **stacked behind
both defects** and could never have cleared from the scope fix alone. The two
diagnoses are the same defect; round 13's accounting of which tests it blocked
was wrong, and the correction is that `slice_of` blocked **23**, not 13.

**Two mechanisms for one idea now sit side by side** and should be looked at
together at some point: `manifest::SurfaceFormCases` holds back a conformance
case whose subject is its byte format, and `embedded.rs`'s `migrate: keep`
marker holds back a Rust-embedded fixture whose subject is its surface form.
They key off different things — a case list versus a comment at the site — so
this is not duplicated logic, but it is one concept with two homes.

### `b36f7a8` — `slice_of` derives what it used to demand

[TYPE-5] names the retained-argument class exactly and `slice_of` is not a
member. `check_slice_of` required a `Targs` node and cited FN-2 when it was
absent — which [DIAG-1] reserves for a user-generic call and never for a table
operation.

**The fix is a deletion, not new machinery.** Both facts were already derived
and then merely re-checked against the written form: the region was read at
`borrow_region` and compared for equality, the element was read from
`check_indexed_place` and compared for equality. Removing the written
arguments removes both comparisons. Each fact now has one source instead of
two that had to agree. A written argument becomes the rejection citing OP-1,
the footing `derivation.rs` already pins for `imin`.

| | before | after |
|---|---|---|
| `slice_of(&'v data)` | FN-2 | **exit 0** |
| `slice_of<'v, u8>(&'v data)` | exit 0 | **OP-1** |
| `len(data)` | exit 0 | exit 0 — de-argumenting was never broken in general |

**The shared-root hypothesis is confirmed by measurement, not inherited.** The
adapter's `fn1-pos-returned-slice-inputs-run` and
`fn1-pos-returned-slice-const-run` both clear from this one fix, nothing newly
failing: **381/7/14 → 383/5/14**. Two independent routes — fixture migration
and the conformance lane — reached one cause and one fix closes both.

**It did not resolve the FN-2/[DIAG-1] question and did not touch it.**
`fn2-neg-eeq-implicit-type` still fails, so the general citation-by-callee-class
rule is unchanged; this removed one wrong FN-2 *site*, not the rule.

`consuming_a_projection_respects_loans_of_residual_fields` and its four
`format!` templates landed here, as ruled — a fixture migration with no
passing test to verify it is what this batch has been avoiding.

**One limitation, and it costs a test.** The derived-element OP-1 branch could
not be shown reachable. Four probes — a non-copy struct element, a generic
element, a nested array element, and an `array_new` of a struct — are each
rejected earlier by TYPE-2 or OP-1 on the array type itself, and **in every
case a control with the `slice_of` line deleted fails identically**. So
`array<T,N>` and `buffer<T>` already require a flat `T` and the element
reaching `slice_of` is flat by construction. The branch is kept because a
source rejection is the correct outcome if it ever is reachable, but it
carries no test, and `slice_of_keeps_nonflat_element_arguments_in_the_op1_domain`
is **left failing and reported** rather than restated onto a violation that
cannot be expressed. The first fixture attempted for it passed for the wrong
reason until the control caught it.

### `b6e89d5`, `b21285e`, `b7cdcfb` — the six ruled restatements

No expectation is edited anywhere in this group.

**(d1) `operation_call_shapes_keep_their_exact_rule_owners`.** Both shapes were
carried on `iadd.wrap`, which [OP-7] moved out of the callee-name inventory, so
it reaches OP-1 at resolution before either shape can be judged. Measured
which shapes still earn the recorded rules rather than guessing: `cvt(value)`
— a retained-argument row with its mandatory arguments absent — earns FN-2
InvalidOperation, and `imin(left: …, right: …)` earns GRAM-11
InvalidNamedArguments. Both rows keep their callee name. The FN-2 assertion is
one of the witnesses that must move if the citation question is settled the
other way, which is what a second witness is for.

**(d4) `region_bearing_buffer_content_rejects_under_stor5`.** Its first
assertion — the written `buffer<slice>` parameter type — was never affected and
is untouched. Its second carried the violation in `buffer_new`'s written
element, which A1 deletes [OP-9]; a region-bearing fill is then caught by the
flat-element requirement citing OP-1 before STOR-5 is reached. **[STOR-5] names
`box_new` and `arena_new` — not `buffer_new` — as the derived-content path it
owns**, and `box_new` derives its content from its operand [STOR-2, OP-2], so
the assertion moves there. Observed: `box_new(move value)` over a slice reports
`Semantics/Source [STOR-5]` at the operand atom the rule names, same
`RegionBearingStorage` kind and mechanical fix. Worth a ruling of its own: for
`buffer_new` the concern is now enforced by OP-1 rather than STOR-5, which is a
stricter gate but a different rule than the one recorded.

**(e1) `complete_role_fixture_…`.** `TypeRegion` came only from a deleted `let`
annotation and now rides a signature-borne `slice<'v, i32>`, the position
[TYPE-5] keeps written. **Fixing that exposed a second loss the first was
masking**: `OperationCallee` is the OPNAME form specifically — `roles.rs:468`
keys it on `TerminalPredicate::OperationName` — and the fixture's only
operation call was `iadd.wrap`, a respelled row that produces no lexical use
at all. It now rides `ineg.trap`, a dotted row that keeps its name. Both losses
were found by running the test, not by reading it.

**(e2) `system_lookalike_…`.** `LexicalUseRole::Type` for `HostString` came
only from `let s: own HostString = …`; the `Construct` use survives the
annotation deletion and the `Type` use does not. A signature produces it —
`fn keeper(value: own HostString)` — so **the escape hatch was not needed and
the role model is intact**.

**(f) the two `OPERATION_FAMILIES` tests.** Re-scoped into two properties.
`every_distinct_op1_family_resolves_through_the_normal_callee_path` runs over
the families that keep a name and asserts the split is exactly 83 − 20, so a
family silently crossing halves cannot pass unnoticed; its identity check is
against each family's own position in the inventory rather than the order it
is written in, so filtering the source cannot make the ordinals agree by
accident. `a_respelled_family_produces_no_lexical_use_at_all` is new and states
the property the respelling introduced, with a named row in the same function
as its control so that the two infix rows being absent is the property and not
an empty search. `dotless_and_dotted_operations_…` keeps its subject — one
dotted, one dotless — on `ineg.trap` and `imin`.

### The seven that remain, each owned elsewhere

| test | owner |
|---|---|
| `driver::…::compiler_independent_negative_cases_…` | (d2) — symptom of the FN-2/[DIAG-1] discrepancy |
| `semantic::…::result_construction_…` | (d3) — folded into `x-give-result-aggregate` |
| `semantic::…::borrows::general_borrows_…` | (c) — `Unsupported { RegionsAndBorrows }` |
| `semantic::…::slices::slice_value_matches_…` | (c) — `Unsupported { OwnershipJoin }`, and it is hiding a negative |
| `semantic::…::slices::slice_of_keeps_nonflat_element_arguments_…` | this round's reported finding: the violation has no v0.23 expression |
| `spec::tests` ×2 | activation-gated by the definition of done |

### Validation

- `make -C compiler check`: **exit 2**, lib **568 passed / 7 failed**, from
  537 / 36 at the rebase and 319 / 253 at round 13's base.
- Adapter **383 / 5 / 14**, identical set before and after the six
  restatements; no conformance case or manifest row was touched at any point.
- `cargo clippy --all-targets -D warnings` exit 0; `cargo fmt --check` exit 0.
- Nothing newly failing at any of the five steps.
## Round 13 (exec-0038n, 2026-08-08) — M3c: the compiler's inline fixtures

Twelve commits on `task/0038-m3c-inline-fixtures`, based on `d89af13`
(verified with `git log --oneline -1`, not taken from the brief). The library
gate moves **319 passed / 253 failed → 533 / 39**, exit 2 both times, read from
`$?` and never through a pipe. **214 tests fixed, zero newly failing** in
either direction of the set diff. The adapter is byte-identical either side —
`Pass=371 Fail=16 Skip=14`, same 16 names — which is the expected result of
not touching the corpus.

Reproduce:

```
make -C compiler check; echo "exit=$?"
cd compiler && cargo test --test conformance --locked --offline -- --ignored --nocapture
```

### One rewriter, reached from Rust — `0bfc70e`, `d6de89a`

`compiler/src/bin/migrate/embedded.rs` adds a `--rust` mode that locates each
Rust string literal, decodes it, hands the bytes to `migrate` **unchanged**,
and re-encodes in the literal form it came from. No second rewriter: every
spelling decision is still the corpus tool's.

Two gates, both load-bearing and both measured rather than assumed:

1. **A literal is rewritten only when the pre-pass changed its bytes.** This is
   what keeps `driver.rs`'s deliberately non-canonical FORM-2 sources
   (`b"fn main() -> own unit pure {}"`, `b" fn main()…"`) and `conditionals.rs`'s
   unflattened `else if` untouched — they parse, so without this gate the
   canonical render would have silently repaired them. This is round 8's
   `form2-neg-noncanonical-ws` hazard, closed by construction.
2. **A rewrite lands only when decoding its own re-encoding reproduces the
   migrated bytes.** An escaping the module cannot round-trip is reported, not
   guessed at.

Eight tests for the scanner and codec, each with its own control: every literal
form located with its content; quotes inside comments and character literals
not mistaken for delimiters; a lifetime not swallowing the literal after it;
`bar` not read as the `b` prefix; escapes decoding to the bytes a test feeds
the compiler; an unmodelled escape refusing rather than guessing; encoding
round-tripping every form; a raw literal refusing content that spells its own
terminator.

The pre-pass gate cannot cover the Bool `match`, which is a spelling class and
a forbidden form at once. A `migrate: keep` comment at the site does, and
records the reason where the next reader is. Two tests for the marker, each
carrying its control: it holds a fixture back where its absence migrates the
same fixture, and its window reaches over the assertion lines a fixture nests
inside but no further.

`compiler/src/bin/migrate/` is excluded from the sweep by hand: its `tests.rs`
fixtures are v0.22 **inputs** and its `rewrite.rs` tables are v0.22 spellings,
both by design.

### What the tool migrated — `fc8d142`, `4cebaea`, `5afd631`

182 Rust files read, 41 changed, 270 fixtures migrated, 3 kept, 29 blocked.
847 annotations, 256 respells, 368 argument lists, 26 constructors written,
0 conditionals (after the markers).

```
cd compiler && find src tests -name '*.rs' | grep -v '^src/bin/migrate/' \
  | xargs ./target/debug/whitefoot-migrate --rust --check
```

Split into three commits by area. Each is **line for line** — semantic 519/519,
backend+lowering 485/485, the rest 107/107 insertions/deletions — which is the
evidence that no fixture was re-laid-out rather than respelled. The tool also
reports changed/total lines per fixture so a whole-fixture re-layout would show
as an outlier; the four highest ratios are all files where nearly every line
carries an annotation or a respell, checked individually.

### The three kept fixtures, each inspected and decided

| site | why it is held back |
|---|---|
| `semantic/tests/conditionals.rs:24` | the Bool `match` **is** the [GRAM-6] rejection under test; migrating it to `if` leaves a source that checks clean |
| `syntax/parser/finalize/tests/corpus_shape.rs:235` | this control **is** the forbidden form the detector detects |
| `semantic/tests/derivation.rs:224` | its violation is the *written* argument `imin<i32>(…)`; A1's deletion, correct on every legal call, removes the violation |

The third was found by measurement, not foresight: it is the **one** test the
mechanical pass newly broke, and it broke loudly rather than silently. `916e882`
restored its bytes and marked it; the tool now reports it kept, and the test
passes again.

### What the tool structurally could not reach — `d726678`, `73dc1c2`, `585cae5`, `d44fd3b`, `8c32713`, `b72d648`

The brief's "lexeme-walking removes the `format!`-placeholder hazard by
construction" is true of a placeholder *inside* a fixture and false of a
fixture that *is* a template. Three idioms never reached the tool:

- **`$PLACEHOLDER` templates** (`floating`, `integer_absolute`,
  `integer_negation`, `integer_extended`, `checked_division`) — `$` does not
  lex, so the literal was skipped as not-Whitefoot.
- **`format!` templates** with `{{`/`}}` and `{name}` — not a Whitefoot program.
- **Fragments** — a `class_arms` arm or a `NEUTRAL_MIDDLE` is a statement list.

Every one was migrated by **assembling it the way its test does**, running
`whitefoot-migrate` over the assembled program, and splitting the result back
at the placeholder boundaries. Nothing was hand-derived. For the whole-template
cases the reversal is verified rather than assumed: re-substituting into the
reversed text must reproduce the migrated bytes exactly, and the substituted
value was checked absent from the template first, so no unrelated occurrence
can be captured.

Two of these needed more than a substitution, and both are the delta showing
through:

- `float_conversion`'s emitter chose `{equality}` between `ieq` and `feq`. Only
  `ieq` respells, so the two destination kinds no longer share a call shape;
  the emitter now builds the whole comparison. **The marker sweep missed this
  file entirely** because its binder is spelled `let success{conversion}` — a
  Rust placeholder inside the identifier. A broader sweep over
  `let [A-Za-z0-9_{}]*: own` is what found it.
- `entailment`'s join fixture already wrote `if`, so no v0.22 class fired and
  the pre-pass gate correctly left it alone — but it wrote the unflattened
  `else { if … }` that [GRAM-6] now rejects. Flattened; identical control flow,
  assertion untouched. Swept the tree for the shape: three hits, the other two
  being `conditionals.rs`'s deliberate negative and a piece of Rust.

Two fixtures were restated, both preserving their assertion **unchanged** and
both verified by observation rather than inference:

- `driver.rs`'s `region.wf` case put its undeclared region inside a `let`
  annotation, which A3 deletes along with the violation. Restated as a borrow,
  which keeps writing its region: `whitefootc` reports `Resolution/Source
  [OWN-3]`, the same stage and rule the row records.
- `resolution::tests::semantic_stage_order_…` declared `fn ieq()` to raise a
  FORM-3 reserved-name rejection. v0.23 shrinks `ReservedLowerNames` by exactly
  the four dotless comparisons — the sibling test
  `respelled_comparisons_leave_the_reserved_name_inventory` asserts precisely
  that — so it reached OP-1 instead. Swapped to `ilt`, which stays named under
  ruling O1: `whitefootc` reports `Resolution/Source [FORM-3]`.

### The failure-set diff, both directions

**Zero tests newly fail.** 214 left the set. Among the tests failing in both
runs, **22 changed their reason**, and every one is the same shape: a fixture
that used to die at *Parsing* because it was v0.22 now reaches a later stage.
None is a rejection that stayed a rejection under a different rule while its
test kept passing — every one of the 22 is a visibly failing test, so nothing
passes for the wrong reason.

```
# per-test reason, with byte offsets and node paths normalized away
awk '/^---- .* stdout ----$/ {…}' <log> | sed 's/ByteOffset([0-9]*)/ByteOffset(_)/g' …
```

### The 39 that remain, per test

Two are the activation-gated `spec::tests` the definition of done excludes.
The other 37 are **not fixture-spelling** and none is this task's:

**(a) `slice_of` loses its written arguments but the checker still demands them
— 13 tests.** The candidate's [TYPE-5] names the retained class exactly —
`cvt`, `reinterpret`, `array_new`, `arena_new`, `finf`/`fnan` — and `slice_of`
is not in it, so A1 deletes its arguments. The compiler then cites **FN-2
InvalidOperation**, which [DIAG-1] reserves for a user-generic call and never
for a table operation. Minimal reproduction with a control that distinguishes
the cause:

```
let view = slice_of(&'v data);        # Semantics/Source [FN-2] InvalidOperation
let view = slice_of<'v, u8>(&'v data);# exit 0 — the form v0.23 deletes
let n = len(data);                    # exit 0 — de-argumenting is fine in general
```

Same root cause as the adapter's pre-existing `fn1-pos-returned-slice-const-run`
and `fn1-pos-returned-slice-inputs-run`. Tests: `semantic::tests::slices` ×9,
`entailment::a_slice_of_carries_its_source_length`, `backend::tests::slices` ×3.

**(b) `if`/`else` branch blocks do not open declaration scopes — 14 tests.**
Two `let`s of the same spelling in the two arms of a `match` are two
declaration events; in the two branches of an `if` they collide. Reproduction
with its control:

```
if flag { let length = 0_u64; } else { let length = 1_u64; }
  # Resolution/Source [TYPE-6] DeclarationCollision
match signal { Stop() => { let length = 0_u64; } Go() => { let length = 1_u64; } }
  # exit 0
```

`semantic::tests::entailment::a_fresh_binding_reusing_an_expired_spelling_inherits_no_stale_fact`
is the same shape and its own comment states the intended behaviour: "each arm
declares its own `j`; the second is a distinct declaration event [ENT-2]". The
migrated `tests/programs/wfgrep.wf` hits it twice (lines 325 and 564, both
`let length = 0_u64;` in different `else` blocks), which is what fails the ten
`cost_shape` tests, `effect_attributes`, and
`semantic::tests::checked_cleanup_edges_…` (`move_through_give` declares
`temporary` in both arms of a `value_if`). Same class as the adapter's
`ent5-pos-join-keeps-common-bound`.

**(c) Pre-existing capability gaps — 2.**
`semantic::tests::slices::slice_value_matches_…` reaches
`Unsupported { OwnershipJoin }` (the adapter's `own5-neg-slice-value-match`);
`semantic::tests::borrows::general_borrows_…` reaches
`Unsupported { RegionsAndBorrows }`.

**(d) The concern died with the deleted bytes, or its citation moved — 4.**
Hazard 4's class; **no expectation was edited**.

- `semantic::tests::operation_call_shapes_keep_their_exact_rule_owners`. Its
  first fixture asserted FN-2 for a bare `iadd.wrap(1_i32, 2_i32)`. Measured:
  **all three named forms now cite `Resolution/Source [OP-1]`** —
  `iadd.wrap<i32>(left:…, right:…)`, `iadd.wrap(left:…, right:…)`, and
  `iadd.wrap(a, b)` alike — because [OP-7]'s one-spelling-per-operation rule
  moves the 20 respelled rows out of the callee-name inventory entirely. The
  test's second assertion (GRAM-11 for named arguments on an operation call) is
  masked by the same OP-1; it is still expressible on a row that keeps its name.
  The first has no v0.23 expression at all, so the test cannot go green without
  removing an assertion. **Needs the same ruling as round 8's finding 2.**
- `driver::tests::compiler_independent_negative_cases_keep_their_semantic_rule`.
  Its hard-coded table demands FN-2 from
  `tests/conformance/cases/fn2-neg-implicit-instantiation.wf`, which now reads
  `let a = 40_i32 + 2_i32;` and **compiles clean (exit 0)**. The manifest row is
  already `status: pending`, so the adapter does not see it — the driver table
  and the manifest disagree. Conformance material; not touched.
- `semantic::tests::result_construction_…` reaches TYPE-5 TypeMismatch, the
  adapter's `x-give-result-aggregate` class.
- `semantic::tests::buffers::region_bearing_buffer_content_rejects_under_stor5`
  reaches Op1 where it demands Stor5 — citation moved, reported not edited.

**(e) Coverage fixtures that lost the only form producing a role — 2.**
Reported rather than restated, because which v0.23 form should now supply the
role is a statement about the resolver's role model, not a spelling.

- `resolution::tests::complete_role_fixture_…`: `LexicalUseRole::TypeRegion`
  came only from `let view: own slice<'r, i32> = …`. A3 deletes it, and no
  other written type in the fixture names a region. Signatures keep their
  written types, so a signature-borne `slice<'r, T>` would restore it.
- `resolution::tests::system_lookalike_…`: `LexicalUseRole::Type` for
  `HostString` came only from `let s: own HostString = HostString();`. The
  construct use survives; the type use does not.

**(f) A test whose premise the respell falsifies — 2.**
`every_distinct_op1_family_resolves_through_the_normal_callee_path` and
`dotless_and_dotted_operations_resolve_by_exact_op1_spelling` build their
source from `OPERATION_FAMILIES`, which is now v0.23 and holds `+wrap`, `+`,
`==` … as family spellings. `  +wrap<i32>(1_i32);` is not a call. Deeper than
the generator: [OP-1] says "infix resolution consults no name domain, and an
operator token is never a declaration, callee IDENT, or OPNAME", so the 20
respelled families **have no lexical use at all** and the tests' shared premise
— that every family resolves through the callee path — is false by design for
them. What these should assert is a decision, not a migration.

**(g) One fixture left unmigrated behind (a) — 1.**
`semantic::tests::slices::consuming_a_projection_respects_loans_of_residual_fields`.
Its four `format!` templates in `semantic/tests/slices.rs` (lines 232, 251, 307,
354) all call `slice_of<'view, u8>`; migrating them cannot make the test pass
while (a) stands, and a fixture migration with no passing test to verify it is
exactly what this batch has been avoiding. They should land with the fix.

### Everything still spelling v0.22 in `compiler/`, and why

```
git grep -n -e 'let [A-Za-z0-9_{}]*: own ' -e '\.\(trap\|wrap\|sat\|checked\)<' \
  -e '\b\(ieq\|ine\|ile\|ige\|eeq\)<' -- 'compiler/*.rs' | grep -v 'src/bin/migrate/'
```

| site | reason |
|---|---|
| `driver.rs` ×3 | deliberate pre-parse negatives (`'Bad`, `"bad\t"`, `1e+`) rejected at Lexing or TerminalClassification; the parser never sees the annotation, and the tool cannot render a source that does not classify |
| `semantic/entailment/flow/sources.rs` ×4, `lowering/builder/probe.rs` ×3 | doc-comment prose describing [ENT-3] fact shapes and the probe's recognized loop, not fixtures. **Stale documentation** — worth a pass, not this one |
| `semantic/tests.rs:420` | the deliberate GRAM-11 negative in (d); the pre-pass respells it into `left: 1_i32 + right: 2_i32`, which does not parse, so it is correctly blocked |
| `semantic/tests/slices.rs` ×20 | the four templates in (g) |
| `src/bin/migrate/` | the tool's own v0.22 test inputs and spelling tables |

The 29 the tool reported blocked reconcile: 13 are the above, and **16 are not
fixtures at all** — Rust literals holding the word `match` in a panic message
(`panic!("… must be the match")`), a keyword table (`Self::Match => b"match"`),
or diagnostic prose, which the Whitefoot lexer reads as the keyword. Each was
read individually; none is Whitefoot source.

### Validation

- `make -C compiler check`: **exit 2** (`$?`, not through a pipe), lib
  **533 passed / 39 failed**, from **319 / 253** at `d89af13`.
- `make check`: **exit 2**, failing at the same compiler step. Earlier stages
  pass: repository invariants, spec append-only, spec archive integrity at 23,
  conformance coverage **128/128 rules, 0 uncovered**.
- Conformance adapter: **`Pass=371 Fail=16 Skip=14` before and after, identical
  failure set** (`diff` of the two name lists is empty).
- `cargo clippy --all-targets -D warnings` exit 0; `cargo fmt --check` exit 0.
- The three activation-gated checks remain red by the definition of done, and
  nothing was written to make them green.

### What the brief got wrong

- **"the failures are overwhelmingly fixtures written in v0.22 spelling."**
  True of 214 of the 253, and false of the rest: 27 of the 39 that remain are
  two compiler defects, (a) and (b), that a fixture migration cannot touch.
  The gate does not reach green from this task alone.
- **The three markers are not the complete set, as the brief warned — and the
  gap was a specific one.** `let [a-z_]*: own ` misses
  `let success{conversion}: own …`, a binder whose name carries a Rust
  placeholder. One whole file (`float_conversion.rs`) hid behind it.
- **"lexeme-walking removes the `format!`-placeholder hazard by construction."**
  It removes the hazard of a placeholder inside a fixture. It does nothing for a
  fixture that *is* a `format!` or `$TYPE` template, which is where 15 of the
  fixtures lived and where all the manual work went.

## Round 11 (exec-0038k, 2026-08-08) — the copy gate, restored from the derived type

Two commits, on the lead ruling of 2026-08-08 that corrected round 10's: the
`Type` child had been doing double duty, and its second duty was [FN-8]'s "own
**copy** value". The compiler was the wrong side of the discrepancy; the spec
text stands.

### The fix

`check_requires` now applies `is_copy_type` to the type `check_statement`
derived one line earlier, reached through the `BindingId` the checked `Let`
carries. No annotation is read and no second derivation exists — the judgment
uses the checker's own answer.

Ordering is deliberate and unchanged where it matters: the subset pass still
runs before `check_statement`, so a malformed clause is still reported ahead of
this. Only an ordinary checking error on the initializer wins over the copy
judgment, which is correct — a type that did not derive cannot be judged.

### Every reachable non-copy family, before and after

One probe per family, `whitefootc --emit-llvm -o /dev/null <file>; echo "exit=$?"`,
with the before column built from `2ccbf4a` (the red-test commit) rather than
reconstructed:

| probe | clause `let` | before | after |
|---|---|---|---|
| `f1-array-new` | `array_new<i32, 4>(0_i32)` | exit 0 | `FN-8 InvalidRequires` |
| `f2-1-checked` | `x +checked 2_i32` | exit 0 | `FN-8 InvalidRequires` |
| `f2-2-checked` | `x -checked 2_i32` | exit 0 | `FN-8 InvalidRequires` |
| `f2-3-checked` | `x *checked 2_i32` | exit 0 | `FN-8 InvalidRequires` |
| `f2-4-checked` | `x /checked 2_i32` | exit 0 | `FN-8 InvalidRequires` |
| `f2-5-checked` | `x %checked 2_i32` | exit 0 | `FN-8 InvalidRequires` |
| `f3-cvt-partial` | `cvt<i64, i32>(x)` | exit 0 | `FN-8 InvalidRequires` |
| **`f5-cvt-total`** | `cvt<i32, i64>(x)` | exit 0 | **exit 0** |
| **`f4-copy-positive`** | `ilt(a, 8_u64)` and `a <= 8_u64` | exit 0 | **exit 0** |

The last two rows are the over-rejection guard and are the reason the gate is
judged on the derived type rather than on the row spelling: a **total** `cvt`
yields a plain `i64`, which is a copy type and stays admitted, while the
narrowing one yields an `Option`/`Result` and does not. A spelling-based filter
could not separate those two — same row family, different derived type.

**Joined, because the two halves argue necessity and sat apart (lead,
2026-08-08).** These measurements are the necessity argument, not a convenience
one, and neither half carries it alone. `array_new` and the `checked` rows show
the admitted-row filter cannot *imply* copy — every one of them is pure, total
and non-trapping, so the vocabulary check passes them and something else must
reject them; that establishes that some copy check is required. The two `cvt`
directions then show a spelling-based copy filter is impossible, since one row
family yields opposite verdicts. Together they say the derived type is the only
thing that can carry this rule — which is why the ruling's original ground, that
the deleted `Type` child owed nothing but typing, was wrong, and why restoring
the check by reading an annotation would have been wrong too.

Six real corpus programs with clause locals also still reach their verdicts
through the adapter, including the two `run` cases and the `trap` case, which is
a stronger over-rejection check than any single probe.

### Measured, against round 10's numbers as the floor

```
$ make -C compiler check; echo "exit=$?"
round 10  exit=2   318 passed; 253 failed
round 11  exit=2   319 passed; 253 failed
```

```
$ cd compiler && cargo test --test conformance --locked --offline -- --ignored --nocapture
round 10  Pass=371  Fail=16  Skip=14
round 11  Pass=371  Fail=16  Skip=14
```

Failure-set diff against round 10, **both directions empty**, for the lib tests
and the adapter alike. The one new passing test is this round's regression.

Then `6b0dd43` adds the conformance case below, taking the adapter to
**Pass=372 Fail=16 Skip=14** with the failure set again unchanged in both
directions.

`make check` exits 2 on the same pre-existing compiler target, every other step
passing, `coverage 128/128` both before and after the new case.

### The gap between the count and the claim

Worth more to a later reader than the numbers around it: **until `6b0dd43`,
neither a conformance case nor a lib test covered a non-copy clause local.**
Round 10's `Pass=371 Fail=16` was honestly measured and was never evidence that
[FN-8] is fully enforced — a rule with no case behind it contributes nothing to
that total whether the compiler checks it or not. The gate was reachable and
unenforced while the corpus was green, which is precisely the shape of defect a
verdict count cannot show. Two things now protect it: the lib test in `2ccbf4a`
and the case in `6b0dd43`.

### The ordering is unobservable, verified rather than assumed

The copy judgment runs after `check_statement` while the subset pass runs
before it, so a clause `let` that is *both* non-copy and outside the subset
changes which check fires. Both cite `FN-8/InvalidRequires` through the same
`invalid_requires(entry)`, so the ruling expected no observable difference —
but the coordinate could still have moved, so it was measured.

`d1-double-illformed.wf` is non-copy *and* carries a subscripted operand:
`let xs = array_new<u64, 4>(ys[0_u64]);`. Its full diagnostic is byte-identical
across the gate:

```
before (2ccbf4a)  Semantics/Source [FN-8]: SemanticIssue { rule: Fn8, location: SourceNode(NodePath { components: [0, 0, 3, 0] }, SyntaxCoordinate { source: SourceId(0), start: ByteOffset(70), end: ByteOffset(108) }), kind: InvalidRequires }
after  (96735a5)  Semantics/Source [FN-8]: SemanticIssue { rule: Fn8, location: SourceNode(NodePath { components: [0, 0, 3, 0] }, SyntaxCoordinate { source: SourceId(0), start: ByteOffset(70), end: ByteOffset(108) }), kind: InvalidRequires }
```

Same rule, same kind, same node path, same byte range. The coordinate does not
move, because both paths report the `requires_entry` rather than the offending
operand. No new diagnostic kind was introduced: a non-copy local cites exactly
what the deleted annotation check cited.

### `6b0dd43` — the corpus coverage, isolated by construction

`fn8-neg-requires-noncopy-local` binds `array_new<u64, 4>(0_u64)` in a clause
and is well-formed in every other respect, so the admitted-vocabulary filter
passes it — `array_new` is pure, total and non-trapping — and only the copy
restriction can reject it. Verified rather than asserted: the same source
**compiles at `2ccbf4a`** and rejects `FN-8` at `96735a5`, so nothing else in
the program is what rejects it. The manifest row records the observed verdict.

### The regression landed first, and red

`2ccbf4a` adds the test one commit ahead of `96735a5`, failing with
`expected Fn8/InvalidRequires, got Complete(...)` — the defect stated as a test
rather than as prose. It carries the two non-copy shapes and the copy-typed
positive control in one place, so a future change cannot restore the gate by
over-rejecting every clause `let`.

### One process note against this round

The copy-gate edit was written, verified green, and then **destroyed** by a
`git checkout <commit> -- <file>` used to measure the before column, because it
had not been committed yet. Re-applied and committed immediately. The rule the
brief gave — commit after every sub-step — exists for exactly this, and the
differential-measurement habit is what collides with it: measuring the before
column requires moving the working tree, so the after state must be committed
*first*. Nothing was lost beyond the rework.

## Round 22 (exec-uninfix, 2026-08-08) — the owner cancels the infix comparisons

Round number 22 rather than 16 because this workspace is based on `726df7f8`,
whose copy of this record stops at round 15; `main` has since reached 21. The
section is written to land above the highest number on the integration branch
rather than to fit this copy, and integration re-anchors it.

Four commits, `726df7f8..ddd12559`:

| commit | what it is |
|---|---|
| `1c891417` | the candidate and its delta amended |
| `36b3271f` | the one rewriter gains the class, and loses four respells |
| `fdc5f0ef` | the corpus and the embedded fixtures migrated |
| `ddd12559` | lexer, tables, catalog, tests, digests |

### The decision, and what it changed in the delta

All six integer comparisons keep their named calls. Arithmetic infix is
untouched, so the batch's spelling rule becomes a grammar class — arithmetic is
infix, comparison is a call — rather than the four-of-six subset the owner
rejected.

**This is the first repair in the delta's history to remove sites.** Two
replacements become byte-identical to v0.22 and therefore stop being
modifications at all: [GRAM-1]'s compound-token sentence (the set stays at two,
`->` and `=>`) and [ENT-3] S1's comparison-origin clause (the six named
comparisons are exactly v0.22's enumeration). Verified by diffing each line
against `spec/kernel-spec-v0.22.md` rather than by reading. The totals go
**64 → 62 sites across the same 34 rules**, operator terminal spellings
**20 → 16**, and `DotlessOperationNames`/`ReservedLowerNames` back to their
exact v0.22 membership — so R2 is **discharged rather than deferred**, and the
R2 widening the delta recorded (four names becoming writer-reusable) dies.
Productions stay at 69: `infix_op` loses four alternatives, not a production.

§2's `expr` EBNF block and [EX-1]'s program block move for the first time in
this document's history. New MD5s recorded in §7:
`c08dbb71b541f5770fff5a249010343d` and `bf6fad0113ea2036aab6ab6c156d8941`.
The `stmt` block is still `00f6095415ba43440367b87d94f06a3e`.

Candidate SHA-256 is now
`5037bd852adc3c1fc623e1b6e1c9b4c209b9cdc927fb2cb3fdf445ac81d791fd`, re-pinned
in `compiler/src/spec.rs`, `spec/derivation/derivation-ledger.md`, and
`tests/conformance/runner.py`.

### What the brief got wrong, recomputed

- **"the candidate names `ieq`/`ine`/`ile`/`ige` at roughly 189 places".** It
  names them at **6** — the candidate had already respelled them away. The
  delta document names them 44 times. Neither figure is near 189, and I could
  not reconstruct a basis that produces it.
  `grep -coP '\b(ieq|ine|ile|ige)\b' governance/spec-evolution/kernel-spec-v0.23-candidate.md`
- **"532 sites across 169 `.wf` files".** Right to the digit, and one of the
  532 is a `==` inside a `doc` string in
  `tests/codegen/cases/bounds/dominating-guard/07-wrong-comparator-negative.wf`
  — a STRING interior, not a token, in a frozen v0.22 corpus. The migration
  target is therefore **531 across 168 files**: `==` 429, `>=` 50, `<=` 45,
  `!=` 7.
  `git grep -o -F -e ' == ' -- 'tests/conformance/*.wf' 'tests/programs/*.wf' | wc -l`
- **"ERR-2, DIAG-1, DIAG-3, FN-4, FN-8 … have sites that mention comparison
  infix".** None of them does. Every one of those sites is about arithmetic
  infix or about `if`, and all five are unchanged by the cancellation. The
  sites that did mention it were GRAM-1, GRAM-5, OP-1 (i) and (iii), OP-2 (c),
  OP-7, ENT-3 S1 and S4, and EX-1.
- **The step order is not executable as written.** Step 3 (lexer and tables)
  before step 4 (corpus) leaves the migration tool unable to read `!=`, because
  `!` is a raw lexical defect the moment the compound token goes. The order run
  was tool → corpus → compiler, and the reason is recorded in `rewrite.rs`.

### The reverse migration, and the three things it was not

`COMPARISON_UNSPELL` is a class in `whitefoot-migrate`, reached from the same
pre-pass as every other class, so the `.wf` corpus and the Rust-embedded
fixtures share one implementation. Unlike every other class it has no callee
token to key on, so the operator is the anchor and both operands are recovered
by walking the atom forms [GRAM-9] admits — place suffixes, a `deref(…)` group,
`move`, and a borrow prefix. A statement keyword is not one of those forms,
which is the boundary a keyword blacklist would have got wrong and which is
asserted in `check`, `if`, and `return` position. The operator is matched on
its **bytes**, not on one token kind, so the class survives the lexer
reverting: `==`, `<=`, and `>=` then arrive as two byte-adjacent punctuation
tokens, and [FORM-2]'s mandatory spacing is what makes that reading
unambiguous. `!=` has no such fallback and its unit coverage is retired with a
recorded reason rather than left to rot.

Separately, `ieq`/`ine`/`ile`/`ige` move from `INFIX_RESPELL` to
`DEARGUMENTED`, so the tool's v0.22 → v0.23 direction now produces
`ieq(a, b)`. That direction stays live for the frozen corpora.

Three findings the run produced that are not the mechanical rewrite:

1. **`semantic/tests/slices.rs`'s `slice_of` case is a fixture whose subject IS
   its written type argument.** The tool would have deleted it and left a
   source that checks clean — the `derivation.rs:224` class exactly. It gains a
   `migrate: keep` marker. It is newer than the last `--rust` pass, which is
   why no marker was there; the marker window is the literal's line plus three,
   and the first placement missed by one line, which the tool reported.
2. **`semantic/tests/requires.rs` had two fixtures written ABOUT the infix
   comparison** — an admitted infix clause computation, and a subscript reached
   only through the infix tail's own atom. Re-spelling them as calls would have
   left two tests passing while testing nothing, so both are re-keyed to
   arithmetic infix (`a *wrap 2_u64`, `a +wrap xs[1_u64]`). `assert_rule` pins
   rule and kind, so the re-key cannot silently change what fires.
3. **`syntax/parser/tests.rs`'s prefix-selection fixture is deliberately
   compressed non-canonical**, and the canonical re-render flattened it — 30/30
   lines changed for one comparison, the one outlier in the changed/total
   ratios the tool reports. That file and `semantic/tests/infix.rs` were
   reverted and given the tool's own rewrite applied to the operand pair alone,
   so both diffs are line for line.

Eight conformance negatives cannot render at all — they are deliberately
non-deriving sources — and carry no comparison, so they are untouched. Nine
template and fragment fixtures are unreachable by the tool for the reasons
round 13 recorded; each carries the same rewrite, derived from the tool's
output shape on real corpus files and verified by the backend tests that
execute them.

### One file that had never been migrated

`tests/programs/raw_deflate_boundary.wf` landed on `main` earlier the same day
in **v0.22 spelling** and had never been through the migration. The whole-file
tool migrated all of it. That is beyond this task's scope and is reported
rather than hidden; leaving it would have left a `.wf` file in the migration
basis that cannot derive under the active grammar.

It was not invisible — it was hidden behind the lib failure. `cargo test` stops
at the first failing target, so at `726df7f8` the integration targets never ran
and two `programs::raw_deflate` tests were failing unobserved. Both pass now.
The canonical-corpus underived count falls **21 → 20** for the same reason.

**The general point, for whoever reads the pinned failure table:** that table
covers the lib only, and while the lib is red every integration target is
unmeasured. `--no-fail-fast` is what makes them visible.

### Validation, exit codes read from `$?` directly

| gate | at `726df7f8` | at `ddd12559` |
|---|---|---|
| `make -C compiler check` | exit 2, lib 569 passed / 6 failed | exit 2, lib 569 passed / 6 failed |
| failure set by name | the pinned six | **identical, both directions** |
| conformance adapter | Pass=384 Fail=4 Skip=14 | Pass=384 Fail=4 Skip=14, same four names |
| `make check` | exit 2 | exit 2; Python stages green (18 structure tests, 128/128 rules) |
| `whitefoot-grammar-tables --check` | green | green — the committed tables are the amended candidate's |
| `whitefoot-grammar` | exit 1, "candidate changes the lexer or source grammar" | exit 1, same message |

The grammar verifier's exit 1 is the documented fail-closed result for a
grammar-extending candidate and is **not** a regression: the same message comes
back from the pre-amendment candidate bytes, run at this branch's tip. The
check that actually verifies this change is the table generator's `--check`.

`cargo test --all-targets --no-fail-fast`, both revisions, since the lib
failure hides everything after it:

- new failures on the branch: **none**
- cleared: `programs::raw_deflate::each_boundary_and_decode_outcome_reaches_its_own_status`
  and `programs::raw_deflate::the_boundary_driver_decodes_a_file_read_through_the_system_path`
- still failing, both revisions: `every_canonical_corpus_file_re_renders_to_itself`
  (20 underived, was 21) and `tests::recorded_chain_ends_at_the_embedded_specification`
  (activation-gated by the definition of done)

No conformance verdict moved. No manifest row was touched.

### Left for the lead

- **`ex1-pos-worked-example.wf` does not match [EX-1]'s normative bytes**, and
  did not at `726df7f8` either: the case's `doc` says "returning from arms"
  where EX-1 says "returning from branches". A case whose whole job is to
  assert EX-1 verbatim is one word off. Pre-existing, outside this scope, and
  it will bite the byte approval.
- **This workspace's copy of this record lacks rounds 16–21.** The rebase must
  re-anchor this section, and the base-revision figures above were measured
  against `726df7f8`, not against current `main` (which reads 572/3 for the lib
  and carries a different pinned set).

## Round 23 (exec-uninfix, 2026-08-08) — EX-1's drifted copy, and the requires-clause `eeq` pincer

Two parts, one commit and one measurement-only result. Base `0a47553e`.

### Part 1 — EX-1 is normative bytes, and the case was one word off

**Normative, established rather than inferred.** [EX-1] lives in a section
titled "Worked example (**normative bytes**)" and its rule text reads "The
following complete program is byte-exact canonical form". [SCOPE-2] accepts a
program iff it satisfies every rule in the document, and EX-1 is a rule. The
digest pin in the delta's §7 is a consequence, not the evidence.

**The case was a verbatim copy, not a superset.** Measured: block 694 bytes /
34 lines, case 690 bytes / 34 lines, two functions each, and `difflib` reports
exactly one changed line — `doc "… returning from arms …"` against the block's
`"… returning from branches …"`, a 4-byte difference that is precisely
`branches` minus `arms`. **The brief's premise that the case "continues with a
second function the spec's block does not contain" is wrong**, and it matters,
because it was the reason given for why the two could not simply be diffed.
They diff cleanly.

The **superset the brief was thinking of is a different case**:
`run-ex1-value-match.wf` extends `main` with a `sign_of` call and a match over
its result, names no EX-1 in its `rules` list, and is documented as
"EX-1-**class** program". It is not required to track the block and must not be
pinned to it.

Which side was wrong: the block. `sign_of` returns from the **branches** of an
`if`/`else` chain after GRAM-6 replaced the Bool `match`; "arms" is the v0.22
word. Fixed by extracting the block and writing the case from it, so the case
is now byte-identical rather than merely edited toward it.

**The drift entered at candidate assembly (`265aeb7`), not at migration.**
`git log -S` shows the block has read "branches" since the candidate was
assembled and never read "arms"; the case has read "arms" since before v0.23.
The corpus migration (`f9efe0d`) rewrote the case's spellings and left the doc,
**correctly** — the tool never touches STRING interiors, by design. Nothing
could have caught this except a comparison, and there is none.

### A checker is possible, and cheap — reported, not built

The brief said not to invent one. It is worth recording that the honest answer
is not "nothing cheap can check it":

| what it needs | verified |
|---|---|
| where the block is | the fenced block after the exact sentence `[EX-1] The following complete program is byte-exact canonical form:` — the **only** fenced block in §19 |
| which case claims verbatim status | derivable from the manifest: exactly **one** row names `EX-1` in `rules` (`ex1-pos-worked-example`), so no filename is hard-coded |
| anything else | no — the comparison is byte equality |

Both halves are already in memory: the spec as `ACTIVE_KERNEL_SPEC_TEXT`
(`compiler/src/spec.rs`) and the case at
`compiler/src/resolution/tests.rs:1175` through `include_bytes!`. No new
plumbing.

Its limits, stated so the check is not oversold: it pins the **copy to the
block**, not the block to reality, and it must not be extended to
`run-ex1-value-match`, whose whole point is to differ.

### Part 2 — the clause pincer, measured on six spellings

Task #22's claim tested directly with `whitefootc --emit-llvm`, sources under
`do_not_scan/eeq/`. **Not disposed** — the disposition is the lead's.

| # | spelling in the `requires` clause | verdict |
|---|---|---|
| a | `eeq(left, right)`, `own` params | **OWN-1** `BareAffineUse` |
| b | `eeq(move left, move right)` | **FN-8** `InvalidRequires` |
| c | `let l = left; let r = right; eeq(l, r)` | **FN-8** `InvalidRequires` |
| d | `eeq(deref(left), deref(right))`, `&'r` params | **OWN-1** `BareAffineUse` |
| e | same with `&uniq 'r` params | **OWN-1** `BareAffineUse` |
| f | a `const` operand of that enum type | **GRAM-3** — a constructor is not a `cvalue` [CONST-2], and a nominal enum is not const-eligible either |

**Control**: `fn8-pos-requires-eeq.wf` — the same clause shape over a *tag-only*
enum — exits 0. So a clause does admit `eeq`; these six rejections are about the
payload, not about `eeq`-in-a-clause.

**No spelling reaches OP-1.** The claim holds for every form I could construct.
It is not a proof over all forms, and it is not stated as one.

**The pincer, exactly.** OWN-1 fires because `eeq`'s operands are consuming —
they are not in OP-1's closed non-consuming list (`len`, `slice_of`'s viewed
place, a subscript base) — and a payload-carrying enum is affine. FN-8 then
rejects the only escape, in its own words: "User-function calls, construction,
**`move`**, borrowing, subscripting, mutation, control flow, allocation, and any
trapping operation are rejected citing FN-8".

### A live diagnostic defect, independent of the disposition

**OWN-1's `mechanical_fix` in a requires clause is advice the enclosing
construct forbids.** Measurement (a) emits `mechanical_fix: "write `move p` for
the affine place"`; measurement (b) is that exact repair, and it is rejected by
FN-8. A writer who follows the compiler's own suggestion is sent from one hard
error to another, and there is no third spelling — d, e, and f close the
remaining routes. This is not about the conformance case and does not go away
whichever way the case is disposed.

### What the lead and owner have to choose between

The case's manifest subject is "A requires clause cannot widen `eeq` to
payload-carrying enums", expected verdict `reject OP-1`. Measured, a clause
**does** reject it — but never through OP-1. Two readings, and the choice is not
mine:

1. **The verdict is wrong, the case is real.** Re-key it to OWN-1. But then its
   subject is affine operands, which the `own1-*` family already covers, and the
   `eeq`-widening claim is no longer what it tests — the subject-shifted class
   from the sweep.
2. **The case is unreachable.** OP-1's payload-enum rejection cannot be
   exercised from a clause at all, so the case as written has no legal form.

**The OP-1 content is not at risk either way.** The sibling
`op1-neg-eeq-payload-enum` reaches OP-1 from a *function body* by moving both
operands — the exact repair a clause forbids — so the domain rejection is
covered. What the clause case would add over the sibling is only "a clause
cannot widen the domain", which is measured true and delivered by two other
rules.

### Gates

`make -C compiler check` and the adapter, at `0a47553e` and at `eb03951`:

| | baseline | after |
|---|---|---|
| `make -C compiler check` | exit 2, lib 572 passed / 3 failed | exit 2, lib 572 passed / 3 failed |
| failure set by name | three | **identical**, both directions |
| conformance adapter | Pass=387 Fail=2 Skip=13 | Pass=387 Fail=2 Skip=13, same two names |

The adapter's two are `fn8-neg-requires-eeq-payload-enum` (this round's part 2,
deliberately left failing) and `own3-pos-outlives-store` (the A3 counterexample).

### Two doc-staleness items routed to the sweep's disposition, not touched

- `run-ex1-value-match.wf:8` — `doc "… returning from arms (return-position
  **match**)."` over a body that is an `if`/`else` chain with no match in it.
- `manifest.jsonl:172` — `ex1-pos-worked-example`'s doc cites "**§16** worked
  example". EX-1 is §19 in v0.22 *and* in the candidate, so the reference is
  stale from an older version and is not v0.23 drift.

### A correction to round 22's own hand-back

Round 22 told the lead that `grep -c` ignores `-o`. **That is GNU grep's
behaviour, not this machine's.** Measured here on a file containing `aa aa aa`:
`-c` → 1, `-oc` → **3**, `-o | wc -l` → 3, under ugrep 7.5.0. The same flags
mean opposite things in the two greps and neither errors, which is worse than
either single reading. `grep -o … | wc -l` is the portable form.

## Round 24 (exec-uninfix, 2026-08-08) — task #22 disposed, on a sibling nobody had compared it to

Continues round 23's measurements. One commit, `4555e61`, plus a disposition
that needs an authority I do not have.

### The measurement that decides it

Round 23 established that no spelling of `eeq` over a payload-carrying enum
reaches OP-1 inside a `requires` clause. That left three candidate
dispositions. **A fourth measurement collapses them to one**, and it is a case
that was sitting beside the subject the whole time:

`tests/conformance/cases/fn8-neg-requires-eeq-integer.wf` is the *same clause,
the same shape, the same bare operands* — `let same = eeq(left, right);` — over
`own u32` parameters, and it reaches **OP-1 `InvalidOperation`** (measured
directly with `whitefootc`). It reaches OP-1 precisely because `u32` is a copy
type, so no OWN-1 candidate forms and the domain judgment is the first thing
that can fail.

**So the concern is already covered in exactly the disputed position.** "A
requires clause does not widen `eeq`'s domain" is asserted, reachably, by the
integer sibling. The payload-enum case is a second instance of the same concern
that cannot be reached, for a reason independent of the concern itself.

### Disposition: retire it. The other two options are refuted by measurement.

- **Restate outside the clause** — refuted for the reason the brief already
  gave: it duplicates `op1-neg-eeq-payload-enum` and adds nothing.
- **Restate onto the requires-subset boundary** — refuted by measurement.
  `eeq(move left, move right)` over `own u32` parameters rejects FN-8
  identically, so the `move` prohibition is **type-independent** and a
  payload-carrying enum would be decorative in such a case. The boundary
  deserves a case; it does not deserve *this* case's operands.
- **Retire it** — the concern is covered in position by the integer sibling,
  and the payload variant is unreachable. Nothing is lost.

The manifest citation is **not** restated to OWN-1, per the ruling: OWN-1 firing
there is a consequence of A1 deleting the type argument, not the case's subject.

**I have not carried the retirement out.** Removing existing conformance
material takes owner agreement and an approval-ledger entry (`CLAUDE.md`,
Specification and test integrity), and this is a removal. The case, its manifest
row, and its verdict are untouched, so the adapter still reports it failing and
the red is still visible. What is delivered is the decision and its evidence.

### What was added instead, which is free and closes a real hole

`fn8-neg-requires-move-operand` — FN-8's prohibition list names "User-function
calls, construction, `move`, borrowing, subscripting, mutation, control flow,
allocation, and any trapping operation", and the corpus carried negatives for
user calls, mutation, control flow, and trapping operations. **`move` had
none.**

The case is a one-token differential and the control is what makes it evidence:

| source in the clause | verdict |
|---|---|
| `let same = ieq(left, right);` | **accepted, exit 0** |
| `let same = ieq(move left, move right);` | **FN-8** `InvalidRequires` |

The operation, the domain, and the clause shape are all legal in both, so the
moved operand is provably the sole violation rather than the one that fired
first. That is the property the payload-enum case could never have: it carried
two violations at once.

### Gates

| | round 23 tip `2d56bb8` | after `4555e61` |
|---|---|---|
| `make -C compiler check` | exit 2, lib 572 / 3 | exit 2, lib 572 / 3, same three names |
| conformance adapter | Pass=387 Fail=2 Skip=13 | **Pass=388** Fail=2 Skip=13, same two names |
| `make check` Python stages | — | 18 structure tests OK; 128/128 rules covered |
| negative exercises | `-44` | **`-45`** |

The `-44 → -45` move is the point of the addition, and it is the packet's §1.1
metric improving by one real negative rather than by a rule merely being named.

### One thing observed and deliberately not chased

`eeq(move flag, True())` inside a clause rejects **FORM-3** at the `True` token,
not GRAM-9. A construct in an atom position is GRAM-9's subject elsewhere, so
the citation looks surprising. It is outside this task, it is recorded here only
so the next reader does not have to rediscover it, and **it is not measured
beyond that one observation** — no claim is made about which rule is correct.
