# 0038 — FLOOR-5 semantic path and corpus migration

This is a temporary live coordination record, not execution authority.

- **Status:** `WAITING` — 2026-08-07 round 1, handed back at a clean boundary:
  the front end is complete and green, the semantic path and migration are
  not started, and four conformance verdicts need a ruling (below).
- **Authority:** owner approval 2026-08-07 (`governance/APPROVALS.md`); the
  fixed delta `governance/spec-evolution/spelling-relief-candidate.md`
- **Owner / workspace:** exec-0038 / `/Users/bytedance/do_not_scan/wf-0038-exec`
  on branch `task/0036-floor5-grammar-and-migration`
- **Base revision:** f80840d (main), branch rebased onto it
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

## Successor brief

Rounds 1–5 of 0036 and this round are discharged; do not re-derive them.
The front end is done. What remains, in order:

1. **FORM-2 for `if_stmt`/`value_if` — the one structural blocker, and it
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
2. **The semantic path**, unchanged from the card: TYPE-5 derivation,
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
