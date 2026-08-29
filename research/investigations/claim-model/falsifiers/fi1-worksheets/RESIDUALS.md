# Sweep completeness, and the wording residuals

## Sweep table, independently enumerated then diffed

I enumerated every hard error and spec-fixed limit from the final rule text
before reading the table: `[IND-1]` verdict; `[IND-2]` position; `[IND-3]` `ine`,
Typing, Vocabulary, three limits at admission, three limits per produced shape
per step; `[IND-4]` 64 paths, clause (b) refusal, clause (e) refusal, pass work;
`[IND-6]`(i) drop; `[IND-7]` 4 terms, 32 slots, step arithmetic, slot discard,
RELAX, certificate discard; `[IND-8]` magnitude, corner minimum; `[IND-10]`
region; 3.8.1 uniqueness. **Twenty-four, one per row, no row missing and no row
spare.** `[IND-5]`, `[IND-8.T]`, `[IND-8.V]`, `[IND-9]` raise nothing, as stated.

The one thing with no row is **clause (d)'s sign test** - see SEAM.md. It is a
second class-(b) decision and it sits directly on rows 9-11, so by the table's
own second structural reading ("is every computation in a row") it earns one.

## Residuals, all wording-level

1. **Row 14 / line 2953 say row 14 is the only class-(b) decision in the
   pipeline and the only row a prover decision moves.** Clause (d)'s sign test is
   a second. Add a row, and replace `[IND-3]`:1997-2002's argument with the
   shared-second-member argument in SEAM.md.
2. **Part (ii)'s "There is no sixth site" undercounts.** Clause (b)'s no-wrap
   side condition does two things - it fills the pair and, at a `set`
   destination, refuses - and only the refusal is listed. The fill is harmless
   (it is (iii)'s empty-to-filled case) but the enumeration is not closed as
   written.
3. **L2's opening sentence is loose.** "The single place where a prover decision
   can move acceptance is clause (b)'s refusal" is false read literally: filling
   a slot moves acceptance too, in the permitted direction. Row 14's own wording
   ("the only row in this table a prover decision moves") is the accurate form.
4. **"A refusal never truncates the pass" vs "ends the check at `[IND-1]`".**
   When a clause (b) refusal is met while substituting the statement's own
   polynomial or a carried witness hypothesis, the check ends, so the produced
   shape sequence is *not* identical at both versions. Acceptance-neutral - the
   weaker version rejects either way - but part (i)'s "the set of `[IND-3]` tests
   performed, and their answers, are identical at both versions" needs the
   carve-out stated.
5. **Four stale sweep-row references survived the renumbering**, all in 3.9.7's
   F-I1e narrative: 4073 "row 7's clearance" (now 14), 4080 "row 11's discard"
   (now 19), 4096 "row 15's corner minimum" (now 23), 4097 "row 12's `RELAX`"
   (now 20). The neighbouring F-I1d bullets *were* renumbered (3 to 6, row 24),
   so the file is inconsistent about whether history uses old or new numbers.
6. **`[IND-3]`'s L4 paragraph omits disequality** from the list of branch
   conditions that yield no polynomial; only `[IND-7]` group (2)'s parenthetical
   carries it. No acceptance consequence - row 3 is scoped to "statement
   relation" - but the list should be complete.
7. **The false-edge path condition is never defined.** `[IND-6]`(i) says "each
   branch condition governing that path"; nothing says the false edge carries the
   negation, and `ieq`'s negation yields no polynomial while `ieq` yields two, so
   two readings give different slot counts and could cross the 32-slot cap
   differently. Text-fixed under either reading, so not a monotonicity break, but
   it is a determinism nit and it is pre-existing.
8. **Clause (a) `a * b` with two literal operands** exceeds the *pass* paragraph's
   "coefficients under `2^64`" (it reaches `2^128`). The figure survives with room
   to spare; the sentence should say so.
9. **`[IND-10]`'s "the region bounds the substitution, and no depth limit is
   needed"** now reads oddly beside row 12; it is true of *depth* and the per-step
   test bounds *size*, but the sentence is what `N6` was built out of and should
   say which of the two it means.
10. **`[IND-8.V]` doubles every figure** (two views). Harmless, unstated.
11. Line 14 of the front matter runs the new paragraph into "F-D4 rewrote the
    three flagship sources..." on one long line. Cosmetic.

## Diff scope, 081043ce..ee730567

One file, +773/-219, read in full. Every hunk is this round or a consequence:
front matter and 0.1; 2.4 properties 1 and 4; 3.8's four laws (new); 3.8.2's
`[IND-3]`, `[IND-4]` per-step and no-truncation sentences, `[IND-6]`(i) and the
group-1 withdrawal; 3.8.4's L1/L2 restatement; 3.9.1 in full - the derivation
header, the pass paragraph, the premise, the bound, the discard, the order
argument, the five-part monotonicity, the regenerated 24-row sweep, the two
structural readings, `N5` and `N6`; 3.9.7's F-I1e entry and the F-I2 charter; 7.2
rows 44-49; 8's B0; 9's D1; 10's Q3; 11.5; 12.4's ledger rows. **Nothing out of
scope.** Worktree clean; HEAD == origin/batch/0106-claim-model-design.
