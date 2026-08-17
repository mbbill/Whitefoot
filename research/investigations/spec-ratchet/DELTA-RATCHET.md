# DELTA-RATCHET — conciseness ratchet, first pass

Status: INTEGRATED, batch 0070 (W5, "the conciseness ratchet"). What remains
here is the measurement, not the delta text.

Base: `spec/kernel-spec.md` at v0.30, SHA-256
`5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1`. Every line
number is a line of that base file. The quoted base sentences were checked
against it byte-for-byte by the one-shot `verify-delta.rs`, deleted at
integration exactly as its own docstring required.

Scope as briefed: ENT-5, ENT-6, ENT-3 (including its S-sub-rules), DIAG-2,
FN-8. Permission rule as briefed: a restatement may become a bare rule
reference **only** where it is equivalent to the referenced rule's own text,
with both sides quoted. Anything with independent nuance stays.

## 1. Headline result

The five rules plus DIAG-1 carry 104,806 B (verified per rule in §4). This pass
proposes **four cuts totalling 547 B, 0.52%**. The briefed target of 15-25% is
not reachable under the stated permission rule, and §5 shows it is not reachable
by dereferencing restatement at all — across the entire 384,593 B spec, the
total mass of cross-rule near-duplicate sentences is 4,203 B (1.1%), most of it
outside these five rules or ambiguous as to which side owns the content.

The premise being tested was DOSSIER §D2, "Restatement instead of reference:
because refs are unchecked at write time, drafting agents restate hub content
defensively." As a *byte* story that premise is measurably false. The six
balloon rules are not inflated by restatement; they are dense original
normative content. What inflates them is subject matter — DIAG-1 alone carries
33 location-form mentions (24 `SourceNode`, 5 `BundleRoot`, 4 `SourceBytes`)
across five diagnostic stages, each attached to its own case — not duplication.

## 2. The four cuts — integrated

R1 (ENT-3.S3), R2 (ENT-3.S12), R3 (DIAG-2), and R4 (ENT-6) are in the v0.31
candidate; the delta text, its quoted base sentences, and their per-cut
accounting live in `spec/kernel-spec.md` and its status line, not here.

## 3. Arguable — left in place, listed as briefed

Every item below is a real overlap that a byte-hungry pass would cut. Each is
left alone because the equivalence needs a judgment I am not authorized to make
silently.

| # | base lines | overlap | why it stays |
|---|---|---|---|
| A1 | FN-8 1211 / PROG-3 1501 | "A false result has the final `check_stmt`'s [OP-5] trap semantics and invokes the body zero times" vs "A false result emits the final `check_stmt`'s exact [OP-5, DIAG-3] trap record, invokes the body zero times, transfers no source owner to it, and follows [EFF-4] without a second cleanup path." | FN-8 line 1204 already declares program start "follows [PROG-3]", so the sentence looks redundant — but PROG-3's version is *stronger* (exact trap record, EFF-4, no second cleanup path). Replacing a weaker statement with a reference to a stronger one changes what FN-8 requires. Owner-visible semantics. |
| A2 | FN-8 1207 / PRV-3 3052 | "An inherited bridge reached through an entry-body call is checked at that call's selected argument and any rejection is instead owned by PRV-2." vs "An inherited leaf reached through an entry-body call remains a call-argument judgment and is owned only by [PRV-2]." | "bridge" vs "leaf" and "checked at that call's selected argument" vs "remains a call-argument judgment" are probably the same claim, but the subject nouns differ and PRV-3 adds "only". |
| A3 | ENT-6 2941 / PRV-2 3017 | "a recursive component with no local protected-leaf seed remains empty" appears in both, near-verbatim. | Mutual duplication: ENT-6 defines the fixed points, PRV-2 owns the demand stratum, and neither text names the other as owner for this clause. Deciding ownership is a design call, and PRV-2 is outside the briefed scope. |
| A4 | ENT-6 2917 / PRV-3 3047 | "… is internal and creates no rejection or caller-visible target" in both. | Different antecedents: "A false bit with no subject datum" vs "An empty parameter set". Not shown equivalent. |
| A5 | DIAG-2 1929 / FN-9 1306 | "A Bq branch additionally carries/retains the B aggregate parent and no same-view Gv parent" — first clause near-identical. | DIAG-2's second clause is strictly more specific (it names the actual-obligation and FN-8 goal parents); FN-9's adds "only the Uq branch". Each carries content the other lacks. |
| A6 | DIAG-2 1951, 1953 / CLM-3 2610, 2611 | Both rules state that the `ClaimLedger` is derived only after success and never read back as acceptance authority, and both list what a strict event discards. | Mutual duplication again, and DIAG-2's discard list has an extra member ("every checked-program-derived tool projection"). |
| A7 | ENT-6 2859 / PRV-3 3040 | "The complete-state base judgment discharges the obligation exactly when the closed complete fact state at that node derives it [ENT-4, ENT-5]." vs "The [ENT-6] complete-state base judgment runs first." | PRV-3 already references ENT-6; this is a correctly-directed reference, not a restatement. Listed only to record that it was checked. |
| A8 | FN-8 1197 / PRV-2 3011 | "A source call to the unlabelled `main` uses this ordinary judgment" vs "… follows this ordinary rule". | **False positive, and instructive.** The two sentences look like duplicates to any similarity measure, but "this" resolves to a different judgment in each rule. Cutting either would be a silent meaning change. |
| A9 | ENT-5 2822, 2827, 2834, 2839, 2848 | Five differently-scoped spellings of the non-reaching exit enumeration: "`return`, `break` to an enclosing loop, or `propagate`'s error edge" / "a `break` edge naming `@l` or any enclosing loop, a `return` edge, or a `propagate` error edge" / "a `break` naming that counted loop or an enclosing loop, …". | The largest genuine repetition inside the five rules, but the five are **not** equivalent to one another — each scopes its `break` set to its own construct (`@l`, that counted loop, an enclosing loop). Collapsing them needs a new defined term, which is new normative text, not a ratchet move. Sized in §5 as the best remaining candidate for a deliberate, owner-approved definition. |

## 4. Per-rule byte accounting

Base sizes measured over `spec/kernel-spec.md` at the pinned digest; a rule's
span runs from its `[ID]` line to the line before the next `[ID]`.

| rule | base lines | base B | proposed B | delta |
|---|---|---|---|---|
| DIAG-1 | 1546-1891 | 42,060 | 42,446 | +386 (DELTA-DIAG1) |
| FN-8 | 1159-1223 | 10,099 | 10,099 | 0 |
| DIAG-2 | 1892-1968 | 11,845 | 11,727 | −118 (R3) |
| ENT-3 (with S1-S12) | 2670-2754 | 11,741 | 11,430 | −311 (R1, R2) |
| ENT-5 | 2771-2856 | 13,617 | 13,617 | 0 |
| ENT-6 | 2857-2955 | 15,444 | 15,326 | −118 (R4) |
| **six-rule total** | | **104,806** | **104,645** | **−161** |
| whole spec | | 384,593 | 384,432 | −161 |

The ratchet's 547 B of cuts and DELTA-DIAG1's 386 B of fence overhead nearly
cancel. Net effect of this entire pass on file size: −161 B, 0.04%.

Two of the five briefed rules — FN-8 (10,099 B) and ENT-5 (13,617 B) — yield
**no** defensible cut at all. Every overlap found in them is in §3: FN-8's two
candidates (A1, A2) both change what the rule requires, and ENT-5's largest
repetition (A9) is five differently-scoped enumerations that are not equivalent
to each other.

## 5. Why the target is unreachable — three independent measurements

All three were run against the pinned base file. Scripts were one-shot mining
aids outside the repository and are not delivered.

1. **Cross-rule near-duplicate mass.** Token-Jaccard over every pair of
   sentences in different rules, whole file: at threshold 0.60 there are 26
   pairs with a combined shorter-side mass of 4,203 B — 1.1% of the spec. Of
   those 26, one is a false positive (A8), five are mutual duplications with no
   named owner (A3-A6), and five involve rules outside the briefed scope
   (notably FN-9 lines 1230-1231 duplicating FN-8 lines 1161-1163, 387 B, the
   single largest cross-rule duplication in the file). What remains is the
   four cuts in §2.
2. **Table conversion arithmetic.** Markdown pipe syntax costs ~10 B per row
   and ~130 B per fence against 22-37 B of removed prose scaffolding per row.
   Every enumerated region in DIAG-1 was costed; none yields a material
   reduction and the three converted in DELTA-DIAG1 cost +451 B.
   `DELTA-DIAG1.md` §4-5 carries the per-region numbers.
3. **Derived-consequence mass.** Sentences opening with Thus / Therefore /
   Hence / Because / In particular / This is / These are, or containing "for
   example", across the six rules: 2,950 B, 2.8%. This is the only remaining
   pool with a plausible content-preserving story, and every sentence in it is
   an individual judgment about what the specification must still say out loud.
   DIAG-1 holds 974 B of it, FN-8 674 B, ENT-5 670 B, DIAG-2 290 B, ENT-3
   342 B, ENT-6 zero.

Ceiling from all three pools combined: under 4% of the six rules, of which
~1% is defensible without semantic judgment. Reaching the DOSSIER's ≤300 KB
target from 384 KB would require deleting roughly 84 KB — about 700 sentences
of original normative content. That is a decision about what the language
specifies, not a ratchet.

## 6. Recommendation

1. Land R1-R4 if the lead wants the ratchet to exist as a working practice; the
   evidence is mechanical and the risk is bounded. Do not expect bytes from it.
2. Retire the ≤300 KB / ≤250 KB targets from the DOSSIER, or restate them as
   what they can actually be: a per-task *retrieval* target served by the
   `--index` path, which the DOSSIER itself already measures at 5-50 KB per
   task. The file-size target has no reachable path and no reader it serves.
3. If a byte reduction is genuinely wanted, the two honest candidates are the
   A9 exit-edge definition (new defined term, owner-approved) and a
   sentence-by-sentence review of the 2,950 B consequence pool. Both are
   semantic decisions and belong in an owner packet, not in a linter ratchet.
4. The largest single duplication in the file (FN-9 1230-1231 against FN-8
   1161-1163, 387 B) is outside the briefed five rules. If a second ratchet
   pass happens, FN-9 — the second-largest rule at 15,868 B — is where to look
   first.
