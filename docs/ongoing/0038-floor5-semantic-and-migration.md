# 0038 — FLOOR-5 semantic path and corpus migration

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
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

Three facts in the brief above do not hold as written and are corrected
here rather than worked around.

- **Branch tip `b0da5f8` does not exist** in this repository
  (`git cat-file -t` fatal). The real tip of
  `task/0036-floor5-grammar-and-migration` was `2ec8248`, carrying exactly
  the three described commits; the reflog shows `b0da5f8` was never a
  value of this ref. Taken over at `2ec8248` and rebased onto main
  `f80840d`.
- **The candidate hash is neither cited value.** The committed candidate
  hashed to `935b9538…` — round 5's own reported bytes — not the
  `bde4a9ef…` the brief cites as pinned. It was also *stale*: it was
  assembled before main's `32e2af4` extended the [OP-1] (iii) anchor, so
  it still carried the doubled "in this version" clause round 5 reported
  and left literal. Re-applying the delta's corrected (iii) replacement,
  which is the only delta change since assembly, gives
  `a92b4513…`. That is the current-delta assembly and what the pins now
  name; no other byte moved.
- **The round-5 blocker is settled by the definition of done above**, which
  rules candidate-stage pinning a recognized state with exactly three
  checks left red until the owner's activation commit.

## Progress

Claimed at f80840d; branch rebased; candidate re-keyed to the current delta.
