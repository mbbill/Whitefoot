# CLAIM-DISSOLUTION AUDIT — 0107

Adjudication of the two-territory claim-dissolution audit. Inputs: side A's
scenario files (`iteration`, 26 scenarios; `valueflow`, 24 scenarios), side B's
dissolutions, and side A's adversarial rebuttals with 22 strengthened variants.
Tree read: `batch/0106-claim-model-design` via `wf-0106-design-wt`, spec **v0.39
ACTIVE**. Every verdict marked *(compiled)* below is the gate-profile
`whitefootc` built from this tree; my own probes are named `s1`–`s23` and are
listed in §6. I re-ran every probe both sides cited: **all 160 reproduce**, so no
verdict here rests on a disputed observation of the compiler.

## 0. Headline

Nineteen of the fifty primary scenarios dissolve in the language exactly as it
stands today, and six more of the strengthened variants do — because each side
had found the answer to the other side's hardest problem and neither connected
them. The audit's central technical result is that **side A's
"overflow-guard tax", which underwrote half its breaks, is not a language gap at
all**: spelling a cursor advance `+checked` instead of `+wrap` publishes the
offset equality the loop head cannot carry, and the fixed-width load, the
two-byte fold and the coupled decoder step all compile today (`s16`, `s17`,
`s18`). This is side A's own valueflow rule — *branch on the term the obligation
names* — applied to side B's iteration territory. Seven irreducible items
remain, and every one of them names a specific missing checker capability rather
than a missing notation.

Counts over the fifty primary scenarios and the twenty-one strengthened variants
that are scenarios (side A's iteration-37 and -38 are attacks on a proposal, not
programs, and are adjudicated in §3.5):

| verdict | primary | variants | total rows |
| --- | --- | --- | --- |
| DISSOLVED-EXISTING | 19 | 6 | 25 |
| DISSOLVED-PROPOSED | 24 | 11 | 35 |
| SETTLED-BY-RULING | 0 | 0 | 0 |
| IRREDUCIBLE | 7 | 4 | 11 rows, **7 distinct** |

No scenario was settled by the world-values ruling: both sides applied it before
writing, and side A's valueflow file records the four candidates it dropped
under it. **One new grammar production survives** (N2, the contract result
datum). Everything else that survives is a publisher.

## 1. The rulings, as I applied them

- **Design space, not corpus.** No verdict below cites corpus absence. Where a
  side argued from corpus frequency I discarded the argument and kept only the
  program.
- **The intent test.** I applied it to *rewrites* as well as to notation: a
  dissolution whose rewrite adds an executable arm no execution can take is not
  a dissolution, because the writer must invent a behaviour the program does not
  have. This is side A's iteration-36 finding and I adopt it as the audit's
  standing test. It is what kills the "add a dominating branch" answer for
  §16 and §23, and what does *not* kill `+checked`, whose `Err` arm is reachable
  in principle and is the honest handling of the case it names.
- **Existing machinery first.** Two productions were withdrawn under it: side
  B's `by k` (N1) and side B's restoration of S8 (P3). One survives as a
  candidate the audit does not need (`rev`, G1).
- **Determinism.** Every surviving proposal is a fixed rule over a finite,
  already-computed state. The two places where an implementation could diverge
  — side B's `*wrap` box image and its shift image — are corrected in §3.
- **A claim is an always-true lemma.** Where a route required a fact that is
  false on some path (side A's r4 sentinel), I did not accept it; where the fact
  is unconditionally true and only the checker loses it, I did.

## 2. The verdict table

`ITER` = iteration territory, `VF` = value-flow territory. "Route" names the
respelling for DISSOLVED-EXISTING and the proposal for DISSOLVED-PROPOSED.
Proposal names are defined in §3; irreducible ids `I1`–`I7` in §4.

### 2.1 Iteration territory

| id | intent sentence | verdict | route | surviving attack |
| --- | --- | --- | --- | --- |
| ITER-01 | Visit every position from the front, stopping early when I find it. | DISSOLVED-EXISTING | counted `for` with `break`, or the `ige` exit test in an ordinary loop (`p13`, `b18`, compiled) | none; the claim must be deleted in the same edit or [CLM-2] refuses it as redundant (`r2`) |
| ITER-02 | Walk one variable-width item at a time until the buffer runs out. | DISSOLVED-EXISTING | `ige` cursor guard (`b18`); for the item's own bytes see ITER-27 | side A's break on the item tail, repaired by `+checked` (`s16`) |
| ITER-03 | Walk this buffer from the last position back to the first. | DISSOLVED-PROPOSED | **P-LOOP**; the guarded descending loop's only residue is the carried bound (`r13`), and its step is machine-checked (`r13b`) | side A broke G1 `rev`'s necessity and its terminal decrement; `rev` is not needed here |
| ITER-04 | Keep taking records until I run off the end, however far each moves me. | DISSOLVED-EXISTING | `ige` guard over a jumping cursor (`b8`) | none |
| ITER-05 | This cursor wraps to the start rather than running off the end. | DISSOLVED-PROPOSED | index as a function of the binder (`n % capacity`) + **P-ROW** `%` image (`b7b`) | none |
| ITER-06 | I emit at most one byte per byte consumed. | DISSOLVED-PROPOSED | **P-LOOP**; the `+wrap 3` escape stride respelled `+checked` removes side A's fabricated length ceiling (`s18`, compiled) | side A's r6a break, repaired |
| ITER-07 | What I have filled and what I have left add up to the whole buffer. | DISSOLVED-EXISTING | delete the derived counter, guard `filled` (`b20`); chunked writes also compile (`r12`) | none |
| ITER-08 | Every byte consumed produces at most one event. | DISSOLVED-PROPOSED | **P-LOOP**; the forty arms meet at the ordinary join | none |
| ITER-09 | Each record says how many fields follow, and I check they fit. | DISSOLVED-EXISTING | the frame check as ordinary control flow (`b8`, no claim) | none |
| ITER-10 | The header promised this stream expands to the buffer I allocated. | DISSOLVED-EXISTING | the promised total *is* the loop's range (`b5`); the written count leaves by a return taken inside the loop (`s9`, compiled) | side A's r9 break, repaired |
| ITER-11 | Process this buffer sixteen bytes at a time. | DISSOLVED-EXISTING | guard the computed limit, inner counted loop `base..limit`; works with a **variable** width and needs no backward `±wrap` rule (`r6`) | none |
| ITER-12 | For each row, visit the columns before the diagonal. | DISSOLVED-EXISTING | already accepted with no claim (`p1`) | none (calibration) |
| ITER-13 | `lo` and `hi` bracket a stretch of the table and the probe is inside it. | **IRREDUCIBLE (I1)** | window half is **P-LOOP**; the midpoint has no route | side A broke P3; `+checked` does not rescue it (`s15`, compiled) |
| ITER-14 | The delay doubles once per round and cannot run past the type. | DISSOLVED-PROPOSED | cap with `imin` (the real backoff) + **P-ROW** `imin` image + **P-LOOP** (`b22`) | none |
| ITER-15 | A word has sixty-four bits, so I cannot shift more than sixty-four times. | DISSOLVED-EXISTING | counted range plus `break` (`b4`); the digit count leaves by a return inside the loop (`s8`, compiled) | side A's r5 break, repaired |
| ITER-16 | I never take more than I have left. | DISSOLVED-PROPOSED | **P-DOM**: an L0 route from `b <= a` to `a -defined b`. The constant-subtrahend case already works today (`s11`, compiled), so the gap is exactly [ENT-6] 3146 | side A's intent-test break upheld: `-defined` in source is the residual promoted to a comparison, and the fix is the route, not the spelling |
| ITER-17 | Each step adds at most 255 and there are at most `count` steps. | **IRREDUCIBLE (I2)** | — | both sides agree; anchored `[OP-2] sum +defined wide` (`s22`) |
| ITER-18 | The total is the factor added once per element, and both stay small. | **IRREDUCIBLE (I3)** | — | both sides agree; anchored `[OP-2] acc +defined factor` (`s23`) |
| ITER-19 | Keep reading from where I left off; the carry always fits. | DISSOLVED-EXISTING | `ige(carry, 4096)` guard; the constant bound means no offset tax | none |
| ITER-20 | Each step strictly shrinks the position and never leaves the buffer. | DISSOLVED-PROPOSED | **P-LOOP**; the callee's `ensures` is the inductive step | none; side A withdrew its own scenario's reading |
| ITER-21 | The loop ran once per position, so afterwards the counter is that number. | **IRREDUCIBLE (I4)** | as literally written it dissolves (use the endpoint); where the count is the binder or binder-derived it dissolves by boundary export (`s8`, `s9`); where it counts matches it does not (`s13`, compiled) | side A broke P2(a); I withdraw P2(a) and keep P2(b) as **P-CLOSE** |
| ITER-22 | The position I remembered is one of the positions I visited. | DISSOLVED-EXISTING | routed `ensures` on a search function returning from inside the loop (`b19`) | side A's argmax and last-match breaks → ITER-33, ITER-34 |
| ITER-23 | The queue only holds work I have room for, and I stop when caught up. | DISSOLVED-PROPOSED | **P-LOOP** with the capacity-checked push (a real check on the push side, not a fabricated arm at the read) | side A's intent-test break applies to the *room-check* route and not to the P-LOOP route |
| ITER-24 | The count never exceeds the capacity, and when it would, I grow. | DISSOLVED-PROPOSED | **P-LOOP** + **P-MONO** (`a <= a + a` / `a <= a * k`) | side A's smuggled-dependency break upheld; P-MONO is the missing publisher, named here (`s19`, compiled) |
| ITER-25 | I read this header two bytes at a time. | DISSOLVED-EXISTING | `+checked 2` on the cursor advance (`s17`, compiled) — no congruence, no stride form, no dead guard | side A's break on side B's out-of-loop probe upheld; the repair is existing syntax, not a rule |
| ITER-26 | Every link in this arena points at a real node or at the end. | **IRREDUCIBLE (I6)** | — | both sides agree, and side B's test is the right one: the fact is true before the loop, so it is not an iteration fact |
| ITER-27 | Read the next four-byte field, stopping when fewer than four remain. | DISSOLVED-EXISTING | `+checked 4` for the cursor advance (`s16`, compiled): the `Ok` arm publishes `next = at + 4`, the frame test then bounds `at`, and all four named offsets discharge | side A's tax finding is confirmed and localized (`s2` accepts, `s3` rejects on the same statements) and then dissolved |
| ITER-28 | Walk this buffer from the last position back to the first. | DISSOLVED-PROPOSED | **P-LOOP** (`r13`, `r13b`) | G1 `rev` is sufficient but unnecessary |
| ITER-29 | I emit at most one byte per byte consumed; escapes are three bytes. | DISSOLVED-PROPOSED | `+checked 3` (`s18`, compiled) + **P-LOOP** | side A's break on side B's fabricated `source_length` ceiling upheld and then repaired |
| ITER-30 | Each lead byte says how far to advance. | DISSOLVED-PROPOSED | the walk itself already compiles (`s14`, compiled: const stride table + `ige` guard); the *coupled* relation needs **P-MONO** for a non-constant stride | side A's "no row publishes it" upheld; P-MONO is the row |
| ITER-31 | The loop tells me how much it did, and I use that number afterwards. | split: DISSOLVED-EXISTING / **IRREDUCIBLE (I4)** | boundary export where the count is the binder or binder-derived (`s8`, `s9`, compiled); no route where it counts matches (`s13`) | side A's break upheld for the match-count half only |
| ITER-32 | `lo` and `hi` bracket the table and my probe is inside the bracket. | **IRREDUCIBLE (I1)** | — | side A's shape-rule objection to P3 upheld in full |
| ITER-33 | The position I ended up keeping is one I visited. | DISSOLVED-PROPOSED | **P-LOOP** + **P-CLOSE**: `best < extent` is pre-loop true under the nonempty guard argmax needs anyway, and the update arm re-derives it | side A's break on the boundary route upheld; the P-LOOP route survives |
| ITER-34 | If I found one at all, the position I remembered is inside the buffer. | DISSOLVED-PROPOSED | **P-LOOP** + **P-CLOSE** with a nonempty early return; or G1 `rev` | **I overrule side A here** (§5.3): `found < extent` is unconditional once the empty buffer returns early, so no disjunction is needed and this is not the convex-join ceiling |
| ITER-35 | When the count reaches the capacity I grow, and the new capacity is larger. | DISSOLVED-PROPOSED | **P-MONO** + **P-LOOP** (`r24b`, `s19`) | side A's break upheld; the publisher is named |
| ITER-36 | I never take more than I have left. | DISSOLVED-PROPOSED | **P-DOM** for §16; **P-LOOP** for §23 | the *ruling* survives and is adopted as the audit's intent test for rewrites (§1) |
| ITER-37 | *(attack on P-LOOP: nested loops)* | proposal defect, repaired | R is the greatest **simultaneous** solution over all loops of the function | upheld in full; the repair is mandatory text (§3.5) |
| ITER-38 | *(attack on P-LOOP: [CLM-2] monotonicity)* | proposal defect, repaired | redundancy is judged at the **converged** R, and the landing change deletes every claim it subsumes | upheld in full (§3.5) |

### 2.2 Value-flow territory

| id | intent sentence | verdict | route | surviving attack |
| --- | --- | --- | --- | --- |
| VF-01 | Pick the table slot this kind of record uses. | DISSOLVED-PROPOSED | **P-COMMIT** (delivery is a value-commit event) + **P-ORDER** | none; side A verified the premise (`r01a`) |
| VF-02 | Widen the cursor; if that overflows fall back to zero. | DISSOLVED-PROPOSED | **P-COMMIT** + **P-ORDER** | none |
| VF-03 | The index is one past the base when narrow, zero otherwise. | DISSOLVED-PROPOSED | **P-COMMIT** + **P-ORDER** | side A's restatement adopted: the computed give gets *the image the `let` form would get*, side conditions included (`r14w`) |
| VF-04 | Walk whichever buffer this mode selects, to its own length. | DISSOLVED-EXISTING | factor the body into a function whose `requires` states the correlation; call it from each arm (`s20`, compiled) | side A broke side B's slice route; **my route survives both** (§5.3) |
| VF-04a | Walk a mode-selected budget of a mode-selected buffer. | DISSOLVED-EXISTING | same factoring; each call site instantiates its own pair (`s20`) | side A's type-system break on the slice route upheld and routed around |
| VF-04b | Fill a mode-selected buffer to a mode-selected budget. | DISSOLVED-EXISTING | same factoring through `&uniq` (`s21`, compiled); [SET-1]'s slice restriction never arises | side A's [SET-1] break upheld against side B's route only |
| VF-05 | The cursor is five now. | DISSOLVED-PROPOSED | **P-COMMIT** at a `set` commit | none; both sides agree this is the largest single hole |
| VF-05a | The stored cursor is five now. | DISSOLVED-PROPOSED | **P-COMMIT**; the grammatical spelling is `let tmp = if …; set r.at = tmp;` (`s10`, compiled — it rejects only because `set` publishes nothing) | side A's "route (1) is empty / hand-written SSA" **overruled** (§5.3): one fresh temporary, no rename cascade |
| VF-06 | Find the first empty slot and then use it. | DISSOLVED-EXISTING | sentinel index (`b06`) | side A's end-position break → VF-06a |
| VF-06a | Find it, then use where it ended. | DISSOLVED-EXISTING | boundary export: return the end position from inside the loop under `ensures ile(stop, extent)` (`s12`, compiled) | side A's break on the sentinel idiom upheld; **the always-true conjunct is not needed** by the boundary route (§5.3) |
| VF-07 | Advance the write cursor by one record per row. | **IRREDUCIBLE (I4)** | the intra-loop index dissolves via **P-ROW**'s multiply box image; the post-loop accumulator value does not | both sides agree; it is I4 in another costume |
| VF-08 | Widen the cursor; bail out to my caller if that overflows. | DISSOLVED-PROPOSED | **P-COMMIT** (the `propagate` normal edge) | side A's break: the *user-call* case is a different mechanism → VF-08a |
| VF-08a | My callee proved this, and I used `propagate`. | DISSOLVED-PROPOSED | **P-PROPAGATE**: strike `propagated` from [FN-9] 1331's exclusion list (`r08a` accepts, `r08b` rejects) | upheld; the repair is one list item on a *different* list than side B edited |
| VF-09 | Give me a buffer of n bytes. | DISSOLVED-PROPOSED | **N2** | side A's reach break upheld: N2 must also widen the *routed* payload → VF-09a |
| VF-09a | Give me a buffer of n bytes, or tell me it did not fit. | DISSOLVED-PROPOSED | **N2** as amended in §3.10 | upheld |
| VF-10 | Find the field's extent, then walk it. | DISSOLVED-PROPOSED | **N2** (`ensures ile(result.stop, room)` and `ile(result.start, result.stop)`) | side A withdrew its own 1285 objection; the fallible variant needs the same amendment |
| VF-11 | Hand the slice to the measuring function and index with what it returns. | DISSOLVED-EXISTING | pass by borrow (`p_ret_len3`) | none; the residue is a [DIAG-1] defect |
| VF-12 | Let the parser tell me where the record starts. | DISSOLVED-EXISTING | `set cursor = parse_index(cursor: cursor, …)` (`b12`) | side A's fallibility break → VF-12a |
| VF-12a | Let a *fallible* parser advance my cursor. | DISSOLVED-PROPOSED | **P-ARM**: align [FN-9] 1335's match-arm set route with 1334's direct-set route (`r12b` rejects, `r12d` accepts on the destination-is-also-the-argument distinction alone) | upheld; the two routes currently contradict each other |
| VF-13 | Step the reader forward, then read the byte it is now on. | DISSOLVED-PROPOSED | **P-PROJ** (delete `projected` from [FN-9] 1336) | none; side A's aliasing attack failed against it (`r13a`), which is the conformance case P-PROJ needs |
| VF-14 | Fold this value into the table. | DISSOLVED-PROPOSED | **P-ROW** (`%`, `/`) | none |
| VF-15 | Take the midpoint of the search window. | DISSOLVED-PROPOSED | **P-ROW** (shift images) | side A's defect upheld: the drafted `min(ha·2^k, max(T))` is not attained under wrap (§3.2) |
| VF-16 | Clamp the requested position to the last valid slot. | DISSOLVED-PROPOSED | **P-ROW** (`imin`/`imax`) | none; the `value_if` expansion compiles (`b16`) and is the *less* declarative spelling |
| VF-17 | Back up by the margin, stopping at the start. | DISSOLVED-PROPOSED | **P-ROW** (saturating rows) | none |
| VF-18 | Index the transition table by the packed pair. | DISSOLVED-PROPOSED | **P-ROW** (`ishl.wrap`, `ior` via the attained `maxor`) | side B's correction of side A's weaker `ior` sketch upheld; side A's symmetric correction of the shift image upheld |
| VF-19 | Address the cell at (row, col) in the flattened grid. | DISSOLVED-PROPOSED | **P-ROW** (multiply box image) as corrected in §3.2 | side A conceded its probe was genuinely out of bounds; its determinism defect in the box image is upheld and fixed |
| VF-19a | Walk a record array whose stride comes from the header. | **IRREDUCIBLE (I7)** | — | `base <= 15·(count−1)` is not a difference bound (`r07b`, compiled) |
| VF-20 | Fold this header two bytes at a time. | DISSOLVED-EXISTING | `+checked 2` on the cursor advance (`s17`) | **N1 loses its sole justifying customer** (§3.11); side A's determinism defect in N1 is a second reason |
| VF-21 | Read as much as fits in the space I have left. | DISSOLVED-EXISTING | branch on the term the obligation names: `match filled +checked want` (`r21b`, `r21c`) | side A broke P10 in both directions; P10 is demoted to optional (§3.2) |
| VF-21a | The same, with a runtime request size. | DISSOLVED-EXISTING | same (`r21c`) | upheld; DESIGN §5.3's erratum reclassifying this family is over-broad |
| VF-22 | Validate the text once, then fold each byte through the table. | DISSOLVED-EXISTING | size the table to the value's type (`s6`, compiled) — same rejections, same outputs, one table | side A's break on side B's *fusion* route upheld: fusion emits before validating |
| VF-22a | Verify the whole input, then emit. | DISSOLVED-EXISTING | same, where the type is byte-wide (`s6`) | upheld against fusion |
| VF-22b | Verify a property narrower than the element type, then use it. | **IRREDUCIBLE (I5)** | — | u32 points validated `< 300` into a 300-entry table: the table cannot be widened to `2^32` (`s7`, compiled) |
| VF-23 | Build the offset table at start-up, then use it. | **IRREDUCIBLE (I5)** | — | both sides agree; side B's element-range component is named and declined for a stated reason |
| VF-24 | Start the cursor at the beginning / walk the two columns together. | DISSOLVED-PROPOSED | **P-COMMIT** (construction image) + **P-ORDER** | side A's reach note upheld: it covers direct constructions only; a factory needs N2 |

## 3. The proposal list

Ten proposals survive. Nine are publishers and add no writer-visible surface;
one is grammar. Two proposals from side B are withdrawn (§3.11).

### 3.1 P-COMMIT — the value-commit image, indexed by operation and destination

**The rule.** [ENT-3] S5 is re-drafted so that one operation committed to one
destination establishes one image *however it is spelled*. The closed list of
value-commit events is: an ordinary `let` initializer; a `set` or `replace`
commit; a `give` delivery edge of a `value_if` or `value_match`; a `propagate`
initializer's normal continuation binding; an [FN-9] selected-return binding;
and a direct [GRAM-8] construction, which establishes `x.f = a` for each field
whose atom is a term or constant of fragment type and `len(x.f) = len(P)` for
each field initialized by `P` or `move P` at array, slice or buffer type.

**Consumed by.** Nothing new: [ENT-6]'s OP-4 and OP-2 goals through [ENT-4]
closure, over terms [ENT-2] 2870(a)/(b) already admit.

**Determinism.** One image per event over one destination; the event list is
closed and syntactic; no search, no new term kind, no new closure rule.

**Intent.** The writer writes nothing. This is the removal of a distinction by
spelling, which `CLAUDE.md` forbids in the compiler and which the specification
currently makes four separate times.

**Dissolves.** VF-01, VF-02, VF-03, VF-05, VF-05a, VF-08, VF-24 — and it is the
absolute prerequisite of P-LOOP, since with `set` mute every loop-head candidate
is deleted on the first round and P-LOOP reduces to today's subtraction exactly
(`b15` accepts, `b15b` rejects on identical arithmetic).

### 3.2 P-ORDER — image, then closure, then kills

**The rule.** On every value-commit edge, take the [ENT-4] closure of the state
*after* the image and *before* that edge's scope-exit and consuming kills, then
close again.

**Consumed by.** Unchanged.

**Determinism.** A reordering of two operations the checker already performs.
[ENT-5] 3120–3127 already fixes exactly this order at the counted preheader, so
this is the specification's own device rather than a new one.

**Intent.** None written.

**Dissolves.** It is load-bearing for VF-02, VF-03 and VF-24, and side B's
monotonicity finding is the reason it is mandatory rather than desirable:
without it, v0.40 as drafted **rejects** `p_vif_both_bare.wf`, a program v0.39
accepts. I record that as the single most important sentence in either
dissolution file.

### 3.3 P-ROW — the operation-table row images

**The rule.** Each row of the operation table publishes its image in the same
change that defines the row, or publishes the empty image explicitly. The rows
this audit needs, for unsigned T:

| row | image | corrections carried |
| --- | --- | --- |
| `a % d`, `d >= 1` | `Z <= r`, `r <= d − 1`, `r <= a` | scoped to unsigned; the signed rows need their own statement |
| `a / d`, `d >= 1` | `q <= a` | as above |
| `imin(a,b)` / `imax(a,b)` | `r <= a`, `r <= b` / duals | — |
| `a -sat b` / `a +sat b` | `r <= a`, `Z <= r` / `r >= a`, `r <= max(T)` | — |
| `ishr.wrap(x,k)`, k literal | `r <= x`; from `x <= hx`, `r <= floor(hx / 2^k)` | — |
| `ishl.wrap(a,k)`, k literal | **must publish the attained maximum of `a·2^k mod 2^w` over `[0,ha]`, not `min(ha·2^k, max(T))`** | side A's u8/`ha=200`/`k=1` case: attained 254, drafted 255. A weaker-than-unique image is a defect under [ENT-1] |
| `ior(a,b)` | `r >= a`, `r >= b`, `r <= maxor(ha,hb)` | side B's `maxor` is the attained maximum; side A's `< 2^k` sketch is weaker and is withdrawn |
| `a *wrap b` | from `a <= ha`, `b <= hb`: `r <= ha·hb` — **and the rule must fix the arithmetic** (mathematical integers, as [ENT-6] 3143 already does for normalization) **and state what it publishes when `ha·hb` leaves the type** | side A's u16/`ha=hb=300` case gives three defensible readings that accept different programs, which [ENT-1] 2835–2836 forbids |

**Consumed by.** [ENT-6] OP-4 / OP-2 through [ENT-4].

**Determinism.** Each image is a finite set of difference bounds against Z or
against an operand, computed from bounds already in the state. `maxor` and the
attained shift maximum are O(width) arithmetic on two constants.

**Intent.** None written; this is the state DESIGN §5.1 names — an operation
exists and its proof behaviour has not been decided.

**Dissolves.** ITER-05, ITER-14, VF-14 through VF-19, and the intra-loop half of
VF-07.

*Demoted.* Side B's P10 (the backward `±wrap` rule) survives as sound but is
**neither necessary nor sufficient** for the family it was proposed for: side A
compiled the `+checked` rewrite that dissolves VF-21 without it (`r21b`,
`r21c`), and showed it produces a storable fact only when the guard's other side
folds to a constant (`r21a`). ITER-11 and ITER-25, its two other claimed
customers, also dissolve without it (`r6`, `s17`). It should be written if a row
audit wants it, but no scenario in this audit requires it, and **DESIGN §5.3's
erratum reclassifying this family from `vocabulary` to publisher is over-broad**.

### 3.4 P-MONO — the unsigned monotonicity image

**The rule.** For unsigned T, at `a + b` and `a * b` where the state establishes
the no-wrap side condition — which the `Ok` arm of `+checked`/`*checked`
establishes by construction, and which an exact `+`/`*` establishes by its own
discharged [OP-2] obligation — publish `r >= a` (and, for `+`, `r >= b`). This
holds with `b` a *term*, which is what distinguishes it from S7's
constant-offset equality.

**Consumed by.** [ENT-4] closure; the fact is the difference bound
`a − r <= 0`.

**Determinism.** One bound per row instance, from a side condition the checker
already decides. No search.

**Intent.** None written.

**Dissolves.** ITER-24, ITER-30, ITER-35. It is the publisher side B smuggled
into ITER-24 without naming, which side A correctly caught (`r24b`, `s19`).

### 3.5 P-LOOP — loop-head retention (side B's P1, with three corrections)

**The rule.** The head state of a `loop_stmt`, and of a counted `for_stmt` over
its closed post-capture state, is today's subtraction **plus the greatest subset
R of the subtracted facts such that every fact of R is derived at every
continuing back edge of that loop**, in the state obtained by flowing the head
state through the body. R is computed by deleting every candidate not so
derived and repeating until no deletion occurs.

Three corrections are mandatory, and each is a sentence:

1. **R is the greatest *simultaneous* solution over all loops of the function**,
   computed by deletion from the full candidate set over the whole body — not
   loop by loop. Side A's `r_p1_nested.wf` is an ordinary nested walk in which
   an inner-first deletion order retains a false fact and admits an
   out-of-bounds read; the per-loop text fixes no order, so two conforming
   implementations differ and one is unsound. This is the only *soundness* break
   found against any proposal in the audit and it must be in the rule's text.
2. **[CLM-2] redundancy is judged at the converged R**, and the change that
   lands P-LOOP deletes in the same change every claim it subsumes. A derivable
   claim is a hard error (`r2`), so a stronger checker rejects programs a weaker
   one accepted; that is not a reason to refuse P-LOOP — migration cost is not a
   design criterion — but the landing must be one change.
3. **P-CLOSE (§3.6) applies at the back edge.** Without it, every candidate
   whose derivation runs through the compiler-owned binder update dies on the
   edge where P-LOOP tests it.

**Consumed by.** Everything downstream; nothing new is asked of [ENT-4].

**Determinism.** The candidate set only shrinks and is finite (a closed DBM over
the function's term pairs, its disequalities and its signed goals); at most one
round per candidate across all loops; the inductive subsets are closed under
union because the body transfer is monotone in the head state, so the greatest R
exists and the deletion converges to it from above. No widening operator, no
fixed point over `FactState`, no choice point, no backtracking. If cost ever
matters a spec-fixed round cap is legal; nothing needs one yet.

**Intent.** The writer writes nothing. This is why it is preferred to a loop
`invariant` clause, which both sides and I reject: an invariant's deletion
changes nothing about what the program does, a verified one needs exactly this
machinery plus the annotation, and an unverified one is an `assume` that W3 and
[ENT-3] 2910 forbid.

**Dissolves.** ITER-03, ITER-06, ITER-08, ITER-14, ITER-20, ITER-23, ITER-24,
ITER-28, ITER-29, ITER-33, ITER-34. Its step obligations are queries today's
checker already answers (`b16`, `b17`, `r13b` — all accept), and side A's one
empirical break against them (`r6a`) is repaired by `+checked` (`s18`), not by
the artificial length ceiling side B's probe carried.

**What it deliberately cannot do.** P-LOOP never invents a bound that did not
hold before the loop. That is what kills DESIGN §9's A1 counterexamples, and it
is also exactly why irreducible **I4** is irreducible: a counter's bound by the
trip count is not a pre-loop fact.

### 3.6 P-CLOSE — close before scope-exit kills (side B's P2 clause (b))

**The rule.** Take each edge's [ENT-4] closure *before* that edge's scope-exit
kills rather than after ([ENT-5] 3095 reverses).

**Consumed by.** Unchanged.

**Determinism.** A reordering of two existing operations. A consequence whose
own terms are still live is true and survives; a fact supported by a dying term
still dies.

**Intent.** None written.

**Dissolves.** ITER-33, ITER-34, and it is a precondition of P-LOOP's back-edge
test. Side A's `r7_closure_vs_kill.wf` shows today's checker kills first and
closes after, so **every** "remember this index" pattern loses its bound, inside
loops and outside them. Side B marked this optional and single-purpose; it is
neither, and I promote it.

*Withdrawn:* side B's P2 clause (a), the counted exit postcondition
`binder = upper_capture`. Side A showed its sole customer is not served, because
P-LOOP can never retain a binder-relative fact: binder facts are not pre-loop
facts and so are not candidates, and the compiler-owned update publishes nothing
on the edge where the test happens. P2(a) has no customer left and should not be
written.

### 3.7 P-DOM — an L0 route for the two-nonconstant `.defined` goal

**The rule.** [ENT-6] gains one normalization route: the goal `a -defined b` at
unsigned T is discharged when the closed state derives `b − a <= 0`; the goal
`a +defined b` is discharged when it derives `a − Z <= c1` and `b − Z <= c2`
with `c1 + c2 <= max(T)`. This decides a goal from bounds already in the state;
it introduces no term.

**Consumed by.** [ENT-6] OP-2 directly.

**Determinism.** A lookup of two existing bounds and one comparison of
mathematical integers. No search. The constant-operand case is already decided
this way today (`s11` accepts exact `remaining - 7_u64` under
`ile(7_u64, remaining)`), so this generalizes an existing route rather than
inventing one.

**Intent.** None written — and that is the point. Today the writer's own
sentence *"I never take more than I have left"* spelled `ile(want, remaining)`
is refused while `remaining -defined want` is accepted (`b3` vs `b3b`). That is
the residual promoted to source, and it is the calibration failure side A
identified. P-DOM lets the writer keep the comparison.

**Dissolves.** ITER-16, ITER-36's first instance, and it is a prerequisite of
P-MONO wherever the no-wrap side condition has two non-constant operands.

### 3.8 P-PROPAGATE — a propagated outcome keeps its pending summary

**The rule.** Strike `propagated` from [FN-9] 1331's exclusion list: a
`propagate` over a user call delivers the callee's verified routed summary onto
the normal continuation binding.

**Consumed by.** [ENT-3] S12, unchanged.

**Determinism.** One summary on one edge; the error edge leaves the function.

**Intent.** None written. The `match` spelling of the identical call already
gets the fact (`r08a` accepts, `r08b` rejects), so publishing for one and not
the other is the `let`/`set` defect on a third construct.

**Dissolves.** VF-08a. Note that this is a *different list* from the one
P-COMMIT edits: side B's P3 covered the arithmetic right-hand side and left the
user-call case, which is the common one, untouched.

### 3.9 P-PROJ and P-ARM — the two boundary write routes

**P-PROJ.** Delete `projected` from [FN-9] 1336's exclusion list: for
`set P = user_call(…)`, P may be any live [ENT-2] term place of the exact result
type formed with field-selection and `deref` projections and no subscript
suffix. Every other condition of 1334 is unchanged, including [OWN-7]
disjointness, which is what keeps `set r.at = f(…, room: len(r.data))` sound.

**P-ARM.** Align [FN-9] 1335's match-arm set route with 1334's direct-set route.
Today `set outer = payload` publishes when the argument was a *different*
binding (`r12d` accepts) and not when the destination is also the argument
(`r12b` rejects) — while 1334's infallible route *requires* the destination to
be the argument. The two routes contradict each other, and the shape they
disagree about is the cursor advance: `cursor` in, `cursor` out.

**Consumed by.** [ENT-3] S12's destination list and [ENT-5] 3074's kill
ordering, both already written.

**Determinism.** A projected place is already an [ENT-2] term with a canonical
spelling and fixed identity; the disjointness test is [OWN-7]'s existing
relation. No search.

**Intent.** Nothing new is written. Both remove a distinction by the spelling of
the destination.

**Dissolves.** VF-13 (P-PROJ), VF-12a (P-ARM). Side A's aliasing attack on
P-PROJ failed in the callee (`r13a`: a function cannot prove an `ensures` over a
parameter length it has just invalidated), which is the conformance case P-PROJ
needs.

*Side effect worth stating:* with P-PROJ and N2, a callee that must write
several correlated places through `&uniq` writes them by returning one struct
and committing its fields, so DESIGN §12's Q3 has no witness left that these two
proposals do not reach.

### 3.10 N2 — the contract result datum (the one new grammar)

**The production.** [FN-9]'s admitted result datum widens: an unrouted clause is
admitted when the written result is `own T` for T one [ENT-2] fragment integer,
**or when it is `own K` for a struct K, `own buffer<T>`, `own array<T, N>`, or
`own slice<'r, T>`**; a clause operand may be that datum carrying
field-selection and `deref` projections whose final selected type is a fragment
type, or `len(D)` where D is that datum so projected at array, slice or buffer
type — the same operand grammar 1285 already gives a *parameter* datum. Every
other 1285 restriction is unchanged.

**Amendment (side A's reach break, upheld).** The same widening must reach the
**routed** payload: [FN-9] 1278's `where T is a fragment integer` restriction on
`ensures when Ok(value: r): …` must widen identically, or N2 covers only the
infallible half of the boundary. Every allocation that can fail returns
`Result<buffer<u8>, E>`, which N2 as drafted does not reach (`r09b` rejects,
`r09a` does not survive parsing). A growable-vector or arena module is exactly
the fallible half.

**Consumed by.** [ENT-3] S12's existing substitution: `len(result)` becomes
`len(buf)`, a term 2870(b) already admits; `result.stop` becomes `s.stop`, a
term 2870(a) already admits.

**Determinism.** One clause still forms exactly one finite typed
RelationTemplate; substitution is the pre-transfer substitution FN-8 already
performs; no new term kind, no new closure rule, no choice point.

**Intent.** The writer already writes `define room = len(deref(src));` on the
*input* side of the same contract block. N2 lets them finish the sentence on the
output side: `ensures ieq(len(result), n);` is the whole content of a factory's
signature, deleting it changes what the function promises, and it names no bound
the checker chose. That is the line between a contract and proof plumbing.

**Dissolves.** VF-09, VF-09a, VF-10, and the residue of VF-24 for constructor
functions.

### 3.11 Withdrawn, and why — recorded so they are not re-invented

- **N1, the chunked counted range `by k`** (side B, valueflow). Withdrawn on
  *existing machinery first*: its sole justifying customer, VF-20's two-byte
  fold, compiles today once the cursor advance is spelled `+checked` (`s17`),
  and ITER-11's block walk compiles with a **variable** width that `by 16` could
  not express (`r6`). Side A's independent determinism defect is a second
  reason: N1's rule text does not fix the arithmetic of its header test, and a
  strength-reduced `binder <= upper − k` underflows for `0..3 by 4` and
  publishes a **false** fact, admitting an out-of-bounds subscript. Side B's own
  iteration dissolver also considered and declined `by k`; the two halves of
  side B disagreed, and the iteration half was right.
- **P3, restoring [ENT-3] S8's midpoint family** (side B, iteration).
  Withdrawn. It recognizes a three-statement source sequence and publishes for
  it, which is capability by **source shape** — the exact term `CLAUDE.md`
  forbids, and which side B's defence dropped from the list it was quoting. It
  refuses interpolation, galloping and ternary search, which have the same
  intent sentence and the same window invariant, on the spelling of one
  division. And it is invoked through spec line 3009's "the day a corpus program
  writes the shape", which is the accreting-list method DESIGN §3.2 exists to
  abolish. ITER-13's midpoint is irreducible (**I1**) and should be recorded as
  such rather than closed by a shape rule.
- **P2 clause (a)**, the counted exit postcondition. Withdrawn: no customer
  (§3.6).
- **G1 `rev`** is *not* withdrawn and is *not* proposed. It passes the intent
  test cleanly — *"walk this buffer from the last position back to the first"*
  is a traversal sentence whose deletion changes what the program does — but no
  scenario requires it: ITER-03/ITER-28 dissolve under P-LOOP (`r13`, `r13b`),
  and its best customer, ITER-34, also dissolves under P-LOOP + P-CLOSE. If it
  is ever written, two things must change from side B's draft: the terminal
  decrement `lower_capture − 1` is unrepresentable at the type's minimum, so the
  exit test must be `lower_capture < binder` evaluated *before* the decrement,
  and the claim that "S11 is unchanged verbatim and no other rule moves" does
  not survive that repair.

## 4. The irreducible list

Seven distinct items. For each: the knowledge that has no home, the program that
witnesses it, what a checker would need — drawn from {path facts, induction,
interprocedural flow, richer vocabulary, quantification} — and what [ENT-1]'s
determinism law says about that need.

### I1 — a probe inside a carried window

**The knowledge.** `lo <= mid < hi`, where `mid = lo + (hi − lo)/2` and `lo`,
`hi` are loop-carried. Binary search is the famous instance; interpolation
search (`lo + scaled(needle)`), galloping search (`lo + step`) and ternary
search (`lo + d/3`) are the same sentence with a different probe.

**Witness.** `b11_midpoint.wf` (compiled): the loop is *deleted* and the window
is supplied as `requires lo < hi`, `hi <= room`, and it still rejects
`mid < len(table)`. `s15` (compiled): the same with the midpoint formed by
`+checked` rather than `+wrap`, so that the no-wrap condition is discharged by
construction — still rejects. The wall is not the loop and not the domain
proof.

**What a checker would need.** **Richer vocabulary.** The chain needs
`mid − lo <= half` together with `half < span` and `span = hi − lo`: three
distinct terms in one relation, which no difference-bound domain and no octagon
holds. P-LOOP carries the window; P-ROW gives `half <= span`; neither reaches
the last step. Nothing in the publisher family can, because a row image is a set
of facts about the *result place* and this fact is about three places at once.

**What the determinism law says.** [ENT-1] 2835–2836 does not forbid this. A
linear-arithmetic domain with a spec-fixed normal form is closed, deterministic
and search-free; what it costs is the closure's complexity and a much larger
soundness bill. What the law *does* forbid is the cheap alternative: the only
closed-form rule anyone has proposed that reaches this fact is a named multi-row
**shape** source (S8), and shape recognition is forbidden by `CLAUDE.md`
independently of [ENT-1]. **So this is the one place where the two governing
rules point in opposite directions, and the owner should decide which one moves
before any batch tries to close it.**

### I2 — an accumulator bounded by its per-step increments

**The knowledge.** `sum <= 255 · i` — a product of a constant with the loop
counter — from which the writer's real goal, that the exact addition cannot
overflow, follows.

**Witness.** `s22` (compiled): `[OP-2] sum +defined wide`, with `count` bounded
by both the buffer length and a literal.

**What a checker would need.** **Induction *and* richer vocabulary**, in that
order of difficulty. P-LOOP cannot reach it by construction: the fact is not
true before the loop (`sum <= 0` is, and is not preserved), and P-LOOP never
invents a bound that did not already hold. A widening fixed point over a *zone*
domain cannot express `sum <= 255 · i` in any iterate, so DESIGN's B6 does not
reach it either. Both are needed together.

**What the determinism law says.** A widening sequence is legal exactly when the
specification fixes it. An implementation-chosen widening — "widen at every
unstable iterate" without a fixed sequence — is precisely the
implementation-chosen heuristic [ENT-1] forbids, and two conforming
implementations would accept different programs.

### I3 — an accumulator bounded by a parameter product

**The knowledge.** `acc <= i · factor`, a product of two non-constant terms.

**Witness.** `s23` (compiled): `[OP-2] acc +defined factor`. Note that this
scenario is constructed so the *contract half works*: the subscript
`i < len(deref(out))` is already discharged today by `requires ile(n, room)`
plus S11, and only the accumulator residue remains. `requires` is the right home
for what is true at entry and the wrong home for what becomes true one iteration
at a time.

**What a checker would need.** Induction plus **richer vocabulary strictly
beyond I2's**: general products, not products with a constant.

**What the determinism law says.** As I2, and worse: nonlinear arithmetic has no
complete decision procedure, so any rule here is a spec-fixed incomplete
fragment, and the fragment must be fixed in the specification rather than chosen
by the implementation.

### I4 — a counter's bound by the trip count

**The knowledge.** After a loop that increments a counter on some iterations,
`count <= n` where `n` is the loop's extent — and, in the EXIT position, that
this survives to the continuation.

**Witness.** `s13` (compiled): count the matches in a scan, then index with the
count — `hits < len(out)`, undischarged. This is the residue of ITER-21,
ITER-31's second half, and VF-07's post-loop value; the three are one item.

**What a checker would need.** **Induction, of the kind P-LOOP is defined not to
do.** The needed fact relates a body-written quantity to the compiler-owned
binder, so it is not a pre-loop fact and cannot be a P-LOOP candidate; and
[ENT-5]'s binder-update kill removes it on the very edge a retention rule would
test it. This is DESIGN's B6 in its smallest honest form: a head state that may
contain facts *invented* by the analysis, which requires a widening operator and
a verified post-fixed point.

**Where the boundary escapes it, and where it does not.** Two thirds of what
looked like this item is not: whenever the reported count *is* the binder or is
derived from it, the loop can `return` from inside itself and [FN-9] verifies
the bound where the binder is live. `s8` (an itoa digit count) and `s9` (a
decompressor's written count) both compile that way, with no claim and no new
rule. What survives is exactly the count that is not a function of the binder.

**What the determinism law says.** A spec-fixed widening sequence over a
spec-fixed domain is legal; an implementation-chosen one is not. B6's three
named gates are the right shape for that bill.

### I5 — a validated element property narrower than the element type

**The knowledge.** "Every element of this buffer is below K", where K is smaller
than the element type's width, established by a validating pass and consumed by
a later use — the shape of every parser, decoder and protocol reader.

**Witness.** `s7` (compiled): u32 code points validated `< 300` in a first pass,
folded through a 300-entry table in a second — `key < len(glyphs)`,
undischarged. The two available answers both fail here. **Fusion** — running the
use under the validating branch (`b22`, compiled) — changes the program: it
emits output for the valid prefix before discovering a bad byte later, which
side A's `r22a` shows is a different program and side B's own standing rule
forbids. **Widening the table to the element type** genuinely dissolves the
byte-wide case (`s6`, compiled, and it is the better program) and is impossible
at `2^32` entries. `p_content.wf` — a runtime-built offset table — is the same
item with the property "every element is a valid index".

**What a checker would need.** **Quantification.** [ENT-2] 2870(a) excludes
subscript suffixes from terms, so no term names an element and no fact can be
*about* one; [ENT-6] 3218 keeps one conservative all-elements component, which
is a permission structure and not a value range; and the boundary surface
([FN-9] 1285, [FN-8] 1238–1239) admits only places, consts, literals and
`len(P)`, so the property is not statable on a contract either.

**One correction the ceiling text must take.** `content` is **not a clean
ceiling clause, it is a partial one**: [ENT-3] S9 2981 already publishes the
declared element range of a named const array, which is why `p_constarr.wf`
compiles and `p_content.wf` does not. A writer holding those two probes can
falsify DESIGN §5.2's current wording. The clause must say which half it means.

**What the determinism law says.** The obvious half-measure — one conservative
interval per indexable place, seeded by `buffer_new`, widened at each element
write, joined by interval join — *is* deterministic and search-free, and is the
exact generalization of S9. It does not close this item, because the fill loop's
element write is a continuing kill and the component is ⊤ at the loop head; it
needs a loop transfer function, which is I4's machinery. A true quantifier is a
different matter: instantiation is a search unless the instantiation set is
spec-fixed, which is why [ENT-1] 2835–2836 is the reason the language has the
vocabulary it has.

### I6 — a quantified data-structure invariant

**The knowledge.** "Every `next` field in this arena is a valid index or the
sentinel" — true before any loop starts, quantified over all elements, and the
justification for every link-chasing walk.

**Witness.** Sketch only; no probe. Anchored by [ENT-2] 2870(a) and by both
sides' agreement. **This is the one entry on the list resting on reading alone.**

**What a checker would need.** **Quantification, and a home that is not a
loop.** Side B's test is the one that makes this mechanical rather than a matter
of taste: *the fact is true before the loop starts, so P-LOOP would happily
retain it if the language could state it* — which proves the missing thing is
not an iteration fact. Its home, if the language grows one, is a type invariant
or a contract over the aggregate.

**What the determinism law says.** A writer-stated invariant that is not
verified is an `assume`, which W3 and [ENT-3] 2910 forbid outright. A verified
one needs quantifier instantiation at every mutation of the structure, which is
a search unless the instantiation set is spec-fixed. **Both sides recommend
rejecting any iteration notation that would swallow this scenario, and I affirm
that without qualification.**

### I7 — a runtime-strided walk

**The knowledge.** `k · stride < len(records)` where the record stride is read
from a header and the trip count is `len(records) / stride` — the first loop
every binary-format reader writes.

**Witness.** `r07b_recwalk.wf` (compiled): `base < len(records)`, undischarged,
with `stride` bounded by a verified `ensures` and `count` computed by division.
Contrast `r07a_stride.wf`, where the trip count is the literal 8 and P-ROW's box
image closes it: **the family splits on whether the count is a compile-time
constant, which is not a distinction any writer would predict.**

**What a checker would need.** **Richer vocabulary.** The box image publishes
`base <= 15 · (count − 1)`, which is a product of a constant with a non-constant
term and therefore not a difference bound; the flattened two-dimensional address
against a dynamically sized grid is the same item, and side B offered it as the
`vocabulary` clause's witness. `r07b` is the better witness because it is
one-dimensional and more common.

**What the determinism law says.** As I3. Note the adjacent shape that already
works and should be named beside the ceiling: computing `cells` first and
allocating `buffer_new(cells, …)` gives `len(grid) = cells` directly from S6.

### 4.1 What is *not* on this list, and why that matters

DESIGN §5.2's **`flow` / convex-join ceiling clause has no surviving witness in
this audit.** Side A offered VF-04's two-if-t shape and then broke side B's
answer to it with three probes; side A offered ITER-34's last match as the
clause's "real customer". Both dissolve: the mode-selected walk, in both the
read and the write direction, dissolves by factoring the correlated body into a
function whose `requires` states the correlation, so that each arm instantiates
it with its own concrete pair (`s20`, `s21`, compiled, body written once); and
the last match dissolves under P-LOOP + P-CLOSE once the empty buffer returns
early, because `found < extent` is then unconditional and no disjunction is
needed. **A contract instantiated per call site is the language's disjunction**,
and that is a general answer to this family, not a trick for these two programs.
The clause may well be true; it needs a witness before it is published as a
permanent ceiling.

## 5. Honest limits

### 5.1 Which verdicts rest on compilation

**Compiled.** Every DISSOLVED-EXISTING verdict in §2 is a compiled program: 160
probes from the two sides, all re-run and all reproducing, plus my own 23. The
irreducible items I1, I2, I3, I4, I5 and I7 each have a compiled witness whose
residual is quoted. The three proposal defects side A found in side B's rule
texts (the shift image, the box image, N1's header arithmetic) are arithmetic
arguments over quoted rule text, not compilations, and I checked each by hand.

**Read, not compiled.** Every DISSOLVED-PROPOSED verdict is by construction a
prediction about a rule that does not exist, and I mark the strongest ones:

- **P-LOOP's reach** (11 scenarios) rests on reading. What is compiled is that
  its *step obligations are answerable by today's checker* (`b16`, `b17`,
  `r13b`, `s18` all accept), which is the empirical claim that matters most and
  which side A's one break against it (`r6a`) does not survive `+checked`.
- **ITER-33 and ITER-34** rest on my own derivation that `best < extent` and
  `found < extent` are pre-loop facts re-derived on the update arm. Nothing
  compiles this.
- **I6** rests entirely on reading and on both sides' agreement.
- **N2's amendment** to the routed payload is compiled only negatively: `r09a`
  fails to parse and `r09b` rejects, so the gap is real; that N2-as-amended
  closes it is a reading.

### 5.2 Where the sides disagreed and I chose

| dispute | side A | side B | my ruling |
| --- | --- | --- | --- |
| the calibration floor | the counted form covers the unit-step case | an `ige` exit test republishes the cursor bound at every head, whatever the stride | **side B**, and side A withdrew; it dissolves five scenarios and is the largest single finding in the iteration territory |
| the "overflow-guard tax" | a structural wall underwriting six breaks | (no response) | **neither**: the tax is real and is *localized* by `s2`/`s3` — byte-identical statements, accepted over a literal-bound allocation and refused over a parameter buffer — and then **dissolved by existing syntax**, by spelling the advance `+checked` (`s16`, `s17`, `s18`). It is not a language gap |
| §13's wall | the carried window | the midpoint | **side B**, confirmed twice (`b11`, `s15`) |
| P3 (restore S8) | a source-shape rule reviving the accreting list | a met retirement condition | **side A**; P3 withdrawn, I1 recorded |
| P2 | (a) has no customer; (b) is essential | (b) is optional and single-purpose | **side A** on both halves |
| VF-04 | a confirmed ceiling; the honest price is a doubled body | package the pair into a slice value | **neither** (§5.3) |
| VF-21 / P10 | `+checked` beats the publisher, which is also insufficient | the backward `±wrap` rule closes it | **side A**; P10 demoted, the DESIGN erratum flagged as over-broad |
| VF-22 fusion | fusion changes the program | fusion is the same program in one pass | **side A** on the general case; the byte-wide case has a different existing answer (`s6`) |
| N1 `by k` | earned, with a determinism defect | the one earned iteration notation | **neither**; withdrawn on existing machinery (`s17`) — and note that side B's *iteration* dissolver had already declined `by k` on the same law |

### 5.3 Where I overruled a landed break

Four places. Each is a compiled program neither side wrote.

1. **ITER-25 and ITER-27** (side A's breaks on side B's §25 and §2). Side A is
   right that side B's probe was not in a loop and that the accepted spelling
   costs a per-iteration test of an impossible condition. But the repair is not
   a rule and not a dead arm: `+checked` on the cursor advance publishes the
   offset equality and both loops compile with no claim, no guard and no
   congruence (`s16`, `s17`). Side A derived this exact rule — *branch on the
   term the obligation names* — in its **other** territory, against P10.
2. **ITER-31 and ITER-10/15's exit halves.** Side A concluded that P-LOOP cannot
   carry a counter and that the only route is a binder-relative counter that is
   wrong on `break` edges. The route it missed is side B's own §22 answer
   generalized: the loop's result leaves by a `return` taken *inside* the loop,
   where the binder is live and [FN-9] verifies the bound (`s8`, `s9`). Only the
   count that is not a function of the binder survives, and that is I4.
3. **VF-04, VF-04a, VF-04b.** Side A broke side B's slice route with three
   probes and I uphold every one of them: a slice is not writable ([SET-1],
   `r04b`) and cannot be a struct field ([STOR-5]/[OWN-3], `r04g`), so a
   container and a correlated number cannot be packaged into one value. But side
   A's conclusion — a confirmed ceiling whose price is the doubled loop body —
   does not follow. Factoring the body into a function whose `requires` *is* the
   correlation writes the body once and compiles in both directions (`s20`,
   `s21`). This is the machinery the owner's law says to try first, and neither
   side tried it.
4. **VF-06a.** Side A showed the sentinel idiom needs an always-true conjunct
   for an end position (`r06f`) and scored the dissolution as an intent-test
   break. The boundary-export route needs no conjunct: return the end position
   from inside the loop under `ensures ile(stop, extent)` and the caller's own
   `igt(stop, 0)` test — which the writer already wrote — closes it (`s12`).
   And **VF-05a**: side A's "route (1) is empty, the alternative is hand-written
   SSA" is answered by `let tmp = if …; set r.at = tmp;` (`s10`), which is
   grammatical, keeps the place's name, and rejects today for exactly one
   reason — `set` publishes nothing — so VF-05a rests on P-COMMIT alone.

### 5.4 Two compiler defects, independently reproduced by both sides

Neither is a design question and both should be filed as such. `contract_define`
over a projected place is rejected `[FN-8] InvalidRequires` while the identical
expression written inline as a `requires` is accepted (`b13_define` vs
`b13_inline`). And `value_if` over `own buffer<u8>` returns
`SemanticUnsupported { feature: OwnershipJoin }` (`b04_select`) — a **compiler
capability limit, which under `AGENTS.md` is not a source-language rejection**
and must not be read as evidence about the fact model.

### 5.5 What I did not do

I did not attempt to order the proposals into batches, and I did not touch
`DESIGN.md`, `CENSUS.md` or `TERRAIN.md`; where this audit contradicts them
— DESIGN §5.2's `flow` clause (§4.1), DESIGN §5.2's `content` clause (I5),
DESIGN §5.3's backward-`±wrap` erratum (§3.3), DESIGN §5.4's ordering of B1
before B6 (§3.1) — I state the contradiction here and leave those files
unedited.

## 6. Probe ledger

My own probes, under
`.../wf-0107-audit/synth/probe/`, all against the gate compiler for this tree.
Both sides' probes are ledgered in their own files; I re-ran all 160 — 68
scenario probes and 92 dissolution and rebuttal probes — and every verdict
reproduces.

| probe | serves | verdict |
| --- | --- | --- |
| `s1_u32_local.wf` | ITER-27 | rejects `at < len(data)` — no guard on the cursor, so the frame test bounds nothing |
| `s2_u32_guard_first.wf` | ITER-27 | **accepts** — `ige` guard first, over a literal-bound allocation |
| `s3_u32_guard_symbolic.wf` | ITER-27 | rejects `p1 < len(data)` — the same statements over a parameter buffer |
| `s4_headroom_axiom.wf` | ITER-27 | **accepts** — four named offsets, with the allocation headroom supplied as a `requires` |
| `s5_parity_headroom.wf` | ITER-25 | **accepts** — the two-byte fold, same route |
| `s6_fulltable.wf` | VF-22, VF-22a | **accepts** — the fold table sized to the value's type |
| `s7_wide_validate.wf` | **I5** | rejects `key < len(glyphs)` — u32 points validated `< 300`, table cannot be widened |
| `s8_itoa_boundary.wf` | ITER-15, ITER-31 | **accepts** — the digit count returned from inside the loop under an `ensures` |
| `s9_inflate_return.wf` | ITER-10, ITER-31 | **accepts** — the written count returned from inside the loop |
| `s10_setfield_via_let.wf` | VF-05a | rejects `here < len(r.data)` — grammatical; only `set`'s missing image blocks it |
| `s11_defined_const.wf` | ITER-16 | **accepts** — the ordering guard *does* discharge exact subtraction for a constant subtrahend |
| `s12_endpos_boundary.wf` | VF-06a | **accepts** — the end position by boundary export, no always-true conjunct |
| `s13_matchcount.wf` | **I4** | rejects `hits < len(out)` — the count that is not the binder |
| `s14_table_stride.wf` | ITER-30 | **accepts** — the const-table stride walk itself compiles; only the coupled relation needs P-MONO |
| `s15_checked_midpoint.wf` | **I1** | rejects `mid < len(deref(table))` — `+checked` does not rescue the midpoint |
| `s16_u32_checked.wf` | **ITER-27** | **accepts** — the fixed-width load with a `+checked` cursor advance |
| `s17_parity_checked.wf` | **ITER-25, VF-20** | **accepts** — the two-byte fold, same route |
| `s18_coupled_checked.wf` | **ITER-29** | **accepts** — side B's coupled step with the `+wrap 3` stride respelled, no artificial ceiling |
| `s19_growth_checked.wf` | ITER-35 | rejects `count - doubled <= 0` — `a +checked a` publishes nothing; P-MONO is the row |
| `s20_modeselect_fn.wf` | **VF-04, VF-04a** | **accepts** — one loop body, the correlation in a `requires` |
| `s21_modeselect_write.wf` | **VF-04b** | **accepts** — the write direction, same route |
| `s22_accum_const.wf` | **I2** | rejects `[OP-2] sum +defined wide` |
| `s23_accum_param.wf` | **I3** | rejects `[OP-2] acc +defined factor` |

Note that `s4` and `s5` are superseded by `s16` and `s17`: the allocation
headroom axiom they stand in for is a real and sufficient fix, but it is a
narrowing spec change to [OP-9]'s domain predicate (for `buffer<u8>` the ceiling
is exactly `max(u64)`, so no headroom exists today) and the `+checked`
respelling reaches the same programs with no rule at all. I keep both probes in
the ledger because together they are the proof that the wall side A found is
real: with a `2^64`-byte buffer the `+wrap` program genuinely *would* be wrong,
which is why the checker is right to refuse it and why the answer is the checked
operator rather than a weaker rule.
