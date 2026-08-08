# 0038 — FLOOR-5 semantic path and corpus migration

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS` — 2026-08-08 round 7. M2 (`4c79e35`), the
  `box_new` repair (`4e68436`), [OP-1] (ii) infix resolution (`3af8478`),
  the derived-nominal sweep (`55aa991`) and the migration tool's first
  class (`16e8432`) are landed. **M3b needs a ruling before it runs**: 20
  conformance cases expect a *pre-semantic* rejection and cannot survive a
  parse-and-render migration at all. See "M3a".
- **Authority:** owner approval 2026-08-07 and the 2026-08-08 rulings
  (`governance/APPROVALS.md`), including the canonical-renderer ruling; the
  amended delta `governance/spec-evolution/spelling-relief-candidate.md`
- **Owner / workspace:** exec-0038f (round 7) / `/Users/bytedance/do_not_scan/wf0038-r7`
  on branch `task/0038-floor5-semantic-and-migration`
- **Base revision:** fb80bb1 (main), already an ancestor; no rebase was owed
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
