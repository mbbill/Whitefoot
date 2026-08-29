# My enumeration, made before reading the sweep table, then diffed against it

Sources read first: 3.8.1, 3.8.2 (`[IND-1]`..`[IND-6]`), 3.9.1's rule box up to
line 2512, 3.9.2 (`[IND-8]`, `[IND-8.T]`, `[IND-8.V]`), 3.9.3 (`[IND-9]`),
3.9.4 (`[IND-10]`). The sweep block (2513-2560) was read only afterwards.

| my item | rule | kind | table row |
| --- | --- | --- | --- |
| undischarged base or step | `[IND-1]` | hard error | 18 |
| leading-position rule | `[IND-2]` | hard error | 1 |
| magnitude `2^127`, statement polynomial | `[IND-3]` | hard error | 2 |
| degree 4, statement polynomial | `[IND-3]` | hard error | 3 |
| 256 monomials, statement polynomial | `[IND-3]` | hard error | 4 |
| same three on the substituted obligation | `[IND-3]`/`[IND-4]` | hard error | 2-4 |
| **`ine` not admitted as a statement relation** | `[IND-3]` | rejection | **absent** |
| **operand without a fragment integer type** | `[IND-3]` *Typing* | rejection | **absent** |
| **subscripted operand / quantification** | `[IND-3]` *Vocabulary fence* | rejection | **absent** |
| **duplicate `bound_stmt` name in one `fn_decl`** | 3.8.1 (`[CLM-1]` 2754 retained) | rejection | **absent** |
| 64 body paths | `[IND-4]` | hard error | 5 |
| clause (e) `set` refusal | `[IND-4]` | hard error | 6 |
| clause (b) `set` refusal | `[IND-4]` | refusal | 7 |
| 4 elimination terms | `[IND-7]` | hard error | 8 |
| 32 slots | `[IND-7]` | hard error | 9 |
| step arithmetic vs `[IND-3]` | `[IND-7]` | removed | 10 |
| unfillable slot content | `[IND-7]` | discard | 11 |
| `RELAX` corner products | `[IND-7]` | no limit | 12 |
| certificate failure / **inability to evaluate** | `[IND-7]` | discard | 13 |
| over-magnitude published constant | `[IND-8]` | not published | 14 |
| corner-minimum arithmetic | `[IND-8]` | no limit | 15 |
| dropped path condition | `[IND-6]`(i) | dropped | 16 |
| same-region restriction | `[IND-10]` | hard error | 17 |

**Four rows the table does not carry.** All four are admission-time rejections
inside `[IND-3]` and 3.8.1. The table's stated scope is "`[IND-4]` through
`[IND-10]`", but it already carries `[IND-2]`'s admission rule as row 1 and
`[IND-10]`'s as row 17 and `[IND-3]`'s three limits as rows 2-4, so the scope it
actually uses is the whole construct and these four belong in it. All four are
syntactic and carry **no** `[ENT-1]` consequence; the defect is the table's
completeness claim ("Every hard error and every spec-fixed limit reachable ...
is enumerated below"), which 3.9.7 then turns into the falsifier target ("a
break is a nineteenth row").

**Attacks on every row classified (a) - all fail, the classification holds.**

- Row 1: position among the loop's body statements; no fact consulted.
- Rows 2-4: I walked clauses (a)-(e) for prover-dependence. (a) is
  unconditional after the F3 deletion; (c) is a type test; (b), (d) and (e) each
  substitute exactly one witness term whatever derives; (d1)/(d2) choose a
  constant in a *hypothesis*, never in `p`; `[IND-6]`(i)'s drop is a slot's
  content. `p` is prover-independent. Cancellation to a zero coefficient is
  syntactic too. No attack.
- Row 5: `[FN-1]` is the *conservative structural* graph, and 3.12.3 keeps
  impossible branches; no derived fact prunes an edge. No attack.
- Row 6: clause selection is by right-hand-side form and the destination's
  binder kind, both syntactic, at a commit the shape rule fixed as visited. The
  one route that used to move it - a prover-widened reach - is closed by the
  shape rule. No attack.
- Rows 8, 9: every count is an output of the shape rule; two constant-bound
  slots stand at a visited (b)/(d)/(e) commit whether or not anything derives,
  the pair's two stand at every visited (b)/(d) commit, the ordered-pair count is
  a function of the elimination-term list, and the path-condition count is one
  per `[IND-3]` polynomial of a syntactic branch condition. No attack.
- Row 17: the region is a syntactic run; "live and uncommitted" is dataflow.
- Row 7 (one-way): the refusal fires on *not derivable* + `set` destination, so a
  strengthening moves it only to admitting. The shape rule keeps the visit set
  from moving with it, so it cannot drag the pass into a row-6 refusal. Holds.

**Discard rows checked in the operative sentence, not only in the table.**

- Row 11 - `[IND-7]`: "A fact offered for a slot that is not an
  `[IND-3]`-normalizable polynomial does not fill it; the slot is **empty**." No
  other sentence calls it an error. OK.
- Row 12 - `[IND-7]`: "**no `[IND-3]` limit applies to `RELAX`** ... `RELAX`
  returns the larger number and the certificate simply fails; that is a discard,
  not an error", and "`RELAX` is total ... and raises no hard error". The old
  "`[IND-3]`'s magnitude limit applies at every step" is gone from this
  paragraph. OK.
- Row 13 - `[IND-7]`: "A certificate a conforming implementation cannot evaluate
  does not succeed; the check moves to the next one." Discard in the operative
  sentence. **But see BREAK-bounds.md: this is the sentence the round's own
  determinism claim rests on being unreachable, and it is reachable.**
- Row 14 - `[IND-8]`: "not published, and the projection is otherwise unchanged;
  publication never raises a hard error". OK, unchanged by this round.
- Row 15 - `[IND-8]`: the corner-minimum has no limit clause anywhere. OK.
- Row 16 - `[IND-6]`(i): "a path condition whose substitution refuses is dropped
  rather than refusing the statement. **Dropping is a decision about a slot's
  content, never about the slot list.**" OK - but `[IND-4]` clause (e) still says
  flatly "the substitution **refuses** the statement", and only clause (b) got
  the "the one route ... is `[IND-6]` clause (i)'s dropped path condition"
  sentence. For a clause (e) `set` commit reached *only* through a branch
  condition the two sentences disagree. See RESIDUALS in the report.
- Row 10 - the semantics is **removal**, and `[IND-3]` carries the matching
  scope sentence ("Their scope is exactly those two polynomials ... They do
  **not** apply to `[IND-7]`'s certificate arithmetic"). Consistent. The defect
  is not the removal; it is the replacement bound.
