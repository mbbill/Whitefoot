# Acceptance run: the real checker against the frozen simulation

Status: falsifier #1 of DOSSIER.md §6, executed 2026-08-07 against the live
v0.22 checker at base revision 9fa3d6d (task 0035). SIMULATION.md was frozen
2026-08-06 and is the falsifier; nothing here was tuned toward it.

**Verdict up front: the prediction held on the two small programs and failed
on the stress case.** utf8parse landed on its predicted buckets exactly.
sha256 landed one claim over. The deflate-dynamic unit predicted 57% of sites
proven outright and delivered 17%, and predicted ~8 claims and needed 21. The
dominant cause is not any of the four drivers recorded at 0034: it is that
[ENT-5]'s loop rule discards every pre-loop fact in any loop containing a
`return`, which is every loop on the deflate path.

## Method

Measured through the compiler, not by reading source. The test-only dark
checker (`check_semantics_dark`) retains each function's complete
`FunctionEntailment` summary — every [ENT-6] obligation disposition with its
rendered residual, and every [CLM-2] claim disposition — instead of rejecting
at the first undischarged site, so one run yields the whole vector.

Two runs per program:

- **baseline**: the corpus source as it stands.
- **claims blinded**: every `claim n: p because "t";` rewritten to
  `let n_blind = band(p, p);` followed by
  `claim n: n_blind because "t";`. [CLM-2] fixes a `band` result as having no
  comparison origin, so the claim establishes no [ENT-3] S3 fact while the
  statement, its runtime check, its [EFF-2] `traps` contribution, and every
  binding use survive unchanged. (Deleting claims outright is not equivalent:
  effect rows must be exactly exhibited, so deletion changes the program.)

A site discharged under blinding is **proven** — no claim support. A site that
flips to undischarged is **claim-supported**. Each claim was additionally
blinded alone, which produced a clean one-to-one site attribution with no
cross-claim interaction.

Buckets are per subscript obligation. v0.22 attaches obligations to subscripts
only, so `buffer_new` size arguments and `check ... else trap` test assertions
— both counted as "trap sites" by SIMULATION.md — carry none here. Where that
shifts a denominator it is stated.

The probe was a temporary in-crate test, run and then deleted; the transform
and the dark-checker entry point above are the whole of it, and it
reconstructs in a few minutes.

## Results

Predicted values are SIMULATION.md's; actual values are the probe's.

| | utf8parse | deflate-dynamic unit | sha256 |
|---|---|---|---|
| obligation sites, predicted | 44 trap sites, 8 assertions | 30 | 9, 1 assertion |
| obligation sites, actual | **33** | **29** | **9** |
| proven (no claim/branch support), predicted | 25 | 17 (57%) | 0 |
| proven, actual | **22** | **5 (17%)** | **0** |
| claims, predicted | 2 | ~8 | 3 |
| claims, actual | **2** | **21** | **4** |
| sites covered by claims, predicted | 11 | ~8 | 8 |
| sites covered by claims, actual | **11** | **24** | **9** |
| branch/guard restructuring, predicted | 0 | 4–5 regions | 0 |
| branch/guard restructuring, actual | **0** | **0** | **0** |
| requires clauses added, predicted | 0 | 5–7 | 0 |
| requires clauses added, actual | **0** | **1** | **0** |
| redundancy advisories | — | — | — |
| advisories, actual | **0** | **0** | **0** |

Per-function actuals for the deflate unit (obligations / proven / claims):
`read_bits` 1/1/0, `emit_byte` 1/1/0, `inflate` 5/0/5, `decode_length` 2/0/1,
`copy_distance` 3/2/1, `build_huffman_table` 10/0/8, `decode_table_symbol`
2/0/2, `store_dynamic_length` 2/1/1, `decode_dynamic` 3/0/3. Restricting to
the dynamic path only, excluding `inflate`'s stored-block dispatch: 24 sites,
5 proven (21%), 16 claims.

### Verdicts

**utf8parse — held.** 33 sites, 22 proven, 11 covered by the 2 predicted
loop-head claims, 0 branches, 0 requires added. The 3-site gap against the
predicted 25 proven is entirely the three `buffer_new` size arguments, which
v0.22 attaches no obligation to; on a like-for-like denominator the prediction
is exact. The `parse` loop's 11 body sites are covered by
`byte_in_source` and `events_behind_scan` through the existing requires
axiom, precisely as predicted.

**sha256 — held, one claim over.** 9 sites, 0 proven, all 9 covered — by 4
claims rather than the predicted 3. The extra claim is not noise and not a
compiler defect: SIMULATION consolidated the extend-loop bound into "ONE
loop-head claim (`16 <= extend_index < 64`)", and that claim is not
expressible. [ENT-3]'s comparison origin admits one comparison call, and
[CLM-2] fixes a `band` result as having none, so a two-sided bound costs two
claims (`extend_index_from_sixteen`, `extend_index_in_schedule`). Otherwise
the prediction is exact, including that no site proves without a claim.

**deflate-dynamic — diverged, and by a lot.** The site count was nearly
perfect (29 actual against 30 predicted). Everything else was not: 5 sites
proven against 17 predicted, a 12-site miss and a drop from 57% to 17%; 21
claims against ~8, a factor of 2.6. Every one of the deflate unit's 24
claim-supported sites is a site SIMULATION either expected to prove outright
or expected to become an honest `Err` branch.

## Divergences and their causes

### The dominant cause, unattributed before this run

**A single `return` anywhere inside a loop body discards every fact
established before that loop.** [ENT-5]'s loop rule removes, at each iteration
head, every fact "having a support member that any kill event (a)–(d)
occurring anywhere inside the loop body, at any nesting depth, may kill". Kill
(d) is "an edge leaving ... the lexical scope of any support binding". A
`return` edge leaves the scope of every binding in the function, so its
presence anywhere in the loop kills everything at the head — including
allocation-length facts that no execution can invalidate.

Isolated by moving one statement and changing nothing else:

- `D1h`: a loop indexing a const table, with one early `return` in the body —
  `ordered_symbol < len(code_lengths)` **undischarged**.
- `D1i`: the identical `return`, hoisted just outside the loop —
  **discharged**.

This is spec-conformant on a literal reading of [ENT-5], and it is not what
SIMULATION's L0 modelled. It also directly refutes one of SIMULATION's named
successes: "Const-table element ranges prove a 'tainted-looking' index —
`code_lengths[code_length_order[code_index]]` is proven at L0 because every
element of the const order table is < 19." The S9 → S5 → S6 chain does work
(`D1c`, `D1d` both discharge that exact site), but in the real
`decode_dynamic` the surrounding truncation path destroys it, and the corpus
carries `ordered_symbol_in_lengths` to buy it back.

Reach: every deflate function that has a loop also returns inside it —
`read_bits`, `decode_fixed_symbol`, `decode_fixed`, `inflate`,
`copy_distance`, `build_huffman_table`, `decode_table_symbol`,
`decode_dynamic`. sha256's three loops contain no `return`, which is why
sha256 matched prediction. `break` does not trigger it (`D1b`, `D1c` both
discharge with a `break` in the loop).

A `return`, `break`, or `propagate` error edge never reaches the next
iteration head, so counting its scope-exit kills in the loop-head state is
avoidable conservatism, and excluding them looks like an [ENT-1]-monotone
strengthening. That is a decision for the owner and lead, not this task.

### The four drivers recorded at 0034

0034 attributed the deflate gap to four drivers. Each was re-checked here
against at least one concrete site with a control that distinguishes the
cause. Note that the four-driver attribution exists only in 0034's report to
the lead; it is not in `docs/done/0034-op4-discharge-and-claim.md` or any
other committed file.

**1. Conservative [ENT-5] loop reading — confirmed, and understated.** The
absence of loop induction is real and is what costs sha256 all 9 of its sites
(`copy_index`, `extend_index`, `round` are all loop counters with no
induction, exactly as SIMULATION predicted at L0). But on the deflate path the
loop rule costs far more than induction would buy, for the return-edge reason
above. Calling it "no loop induction" understates it: L1 loop induction alone
would not recover `ordered_symbol < len(code_lengths)`, because the fact being
lost is `len(code_lengths) = 19`, an allocation fact that needs no induction.

**2. No `band`/`bor` facts — confirmed.** Concrete site: `decode_length` in
`raw_deflate_dynamic.wf` guards with
`bor<Bool>(ilt(symbol, 257), ige(symbol, 286))` and matches on the `bor`.
[ENT-3](b) excludes `bor`, so the `False()` arm establishes nothing and both
`length_bases[length_index]` and `length_extras[length_index]` need
`length_symbol_in_tables`. The corpus's own justification text already says so
("the bor keeps its bounds out of the fact state"). Control: `copy_distance`,
same file, guards the same way but with a single `ige<u64>(distance_symbol,
30_u64)` scrutinee, and its two const-table sites discharge with no claim —
2 of its 3 sites are in the proven bucket. Micro-controls `D2a` (single
comparison, discharged) and `D2b` (`bor` of two comparisons, undischarged)
reproduce it on four lines. This driver is the reason SIMULATION's "6 of 17
proven sites are proven by pre-existing doctrine-shaped guards" did not
survive: the guards are there, but the two-sided ones are shaped as `bor`.

**3. FN-8 one-final-check threading — confirmed.** Concrete site:
`store_dynamic_length` in `raw_deflate_dynamic_decode.wf`. Its one requires
clause (`literal_count <= len(literal_lengths)`) discharges SITE 0 through S4
plus the dominating `ilt(position, literal_count)` branch — the threading
mechanism works. SITE 1 (`distance_index < len(deref(distance_lengths))`)
needs a second precondition, and [FN-8] admits exactly one final `check`.
Folding two conditions into that one check with `band` establishes nothing,
for the same reason as driver 2: control `D3a` (one comparison in requires,
discharged) versus `D3b` (two folded with `band`, undischarged). So a function
can thread at most one precondition fact, which is why the migration added 1
requires clause where SIMULATION predicted 5–7 — not because the threading tax
was lower than feared, but because the mechanism caps out at one.

**4. No mask-bound source — confirmed, with a correction.** Concrete site:
`decode_dynamic`'s `literal_count = iadd.wrap(read_bits(count: 5), 257)`.
`read_bits` returns a masked value; [ENT-3] has no fact source for `iand`, and
[ENT-3] states outright that no `ensures` exists in this version, so
`read_count` is formally unbounded, S7 declines (it needs a derived range on
the operand), and the wrap-family chain SIMULATION described never starts.
The correction: SIMULATION proposed fixing this at L2 with `ensures result <
1 << count` on `read_bits`, "derivable inside its body from the mask op".
Controls `D4a` (mask in a callee) and `D4b` (the identical mask inline in the
caller) are **both** undischarged, so the missing derivation is not the call
boundary — v0.22 cannot derive a bound from `iand` at all. An `ensures`
construct would supply the fact to callers but could not itself be discharged
in `read_bits`'s body without a new mask fact source. The L2 step is one step
short as specified.

### Predictions that failed outright

**"The L3 residue is exactly 3 branch regions ... As branches they cost one
compare each and buy honest `InvalidHuffmanCode` Err paths where zlib trusts
its own table construction."** Refuted. All three named sites exist and all
three are **claims**, not branches:
`decode_table_symbol`'s `symbols[ordered]` is `ordered_in_symbols`;
`build_huffman_table`'s second-pass `offsets` and `symbols` writes are
`order_slot_in_offsets` and `destination_in_symbols`. A claim aborts; the
predicted honest `Err` path does not exist. Zero branch restructuring was
added to any of the three programs in the 0034 migration — the diff adds only
`let` bindings, `claim` statements, and one requires `check`. Whether the
migration should have spent branches rather than claims at those three sites
is a live question this run does not settle, but the design's claim that the
mechanism *produces* honest error paths is, on this corpus, not what happened.

**"Taint saturation ... did not materialize: zero false positives."** Not
re-testable here. v0.22 has no taint judgment ([ENT-3] says so explicitly), so
the prediction has no referent in the shipped fragment.

### Predictions that held for a reason other than the predicted one

**"Every claim in all three programs is 1–2 comparisons, one line. No
multi-clause residual appeared."** Held, but it is not evidence: [ENT-6] fixes
the residual rendering to exactly the offset's source bytes, ` < len(`, the
base's source bytes, `)`. A multi-clause residual is unrepresentable in
v0.22, so this prediction cannot fail. The falsifiable form is the size of the
*fix*, which was measured from the migration diff and does hold: utf8parse 2
claims + 2 `let`s, sha256 4 + 4, the deflate unit 21 + 39 — a mean of about
two lines per fix, no fix larger than three.

## Findings

1. **The simulation was run against a stronger L0 than the one specified.**
   SIMULATION's L0 assumed "path-sensitive dominating branch/match facts" with
   no comparison-origin restriction and "linear arithmetic (transitivity, ±
   constants, halving)". Shipped v0.22 restricts branch facts to a single
   comparison call or a `let` of one, has no halving, and discards pre-loop
   facts on any loop with a `return`. Three of the four confirmed drivers, and
   the dominant unattributed one, are all instances of shipped-L0 being a
   proper subset of simulated-L0. The design was not wrong about what a
   fragment of that strength would prove; the fragment that got specified is
   weaker than the one that got simulated.

2. **The gap is concentrated, not diffuse.** One rule — the loop rule's
   treatment of scope-leaving edges — accounts for the majority of the deflate
   miss, and it has a candidate monotone fix. The `bor` restriction accounts
   for most of the rest and has a candidate monotone fix (admit `band`/`bor`
   of comparisons as a conjunction/disjunction of relations). Neither requires
   loop induction, `ensures`, or struct invariants.

3. **The claim mechanism does work.** Every one of the 27 claims in the three
   programs is load-bearing: blinding any single claim undischarges at least
   one site, and no claim drew a [CLM-2] redundancy advisory. There is no
   over-claiming in the migrated corpus.

4. **Programs without error paths in loops behave as designed.** sha256 and
   utf8parse both landed on prediction. The design's model is accurate for
   straight-line and clean-loop code and breaks down exactly where real I/O
   shaped code lives.

5. **Claims are silently replacing branches.** The one place the design
   promised an honest recoverable error path — the canonical-Huffman
   well-formedness residue — became three aborts. Nothing in the toolchain
   flagged that substitution.

## Caveats

- The proven bucket is defined as "discharges with every claim blinded". A
  site counted as claim-supported might be provable through some *other*
  claim; the measurement answers "does this site need claim support", not
  "does it need this claim".
- Static dispositions only. No dynamic check counts were measured, and the
  codegen-fusion question of DOSSIER §4.5 remains untested.
- SIMULATION's denominators include `buffer_new` size arguments and test
  assertions, which v0.22 attaches no obligation to. Every site-count
  comparison above is stated on the v0.22 denominator with the difference
  named.
- The deflate unit here includes `inflate`, whose stored-block path
  SIMULATION excluded; the dynamic-path-only subset is reported alongside.

## Pre-activation v0.24 candidate rerun (2026-08-09)

This is review evidence, not an activation record. It was run through the same
dark checker and claim-blinding method against the unapproved v0.24 review
candidate prepared by commit `00e6ce4`, whose complete specification SHA-256 is
`53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`.
The temporary in-crate probe was deleted after the run.

| | utf8parse | deflate full unit | deflate dynamic path | sha256 |
|---|---:|---:|---:|---:|
| obligation sites | 33 | 29 | 24 | 9 |
| proven with all claims blinded | **22** | **11** | **11** | **0** |
| claim-supported | 11 | 18 | 13 | 9 |
| baseline-undischarged | 0 | 0 | 0 | 0 |

The frozen utf8parse and sha256 buckets did not move, so no previously proven
site regressed. Deflate recovered six proven sites: 5 to 11 on both the
29-site full denominator and the 24-site dynamic-path denominator. The focused
`D1h` witness now discharges, while `D1i` remains discharged.

Per deflate function, in `obligations / proven / claim-supported` form:
`read_bits` 1/1/0, `emit_byte` 1/1/0, `inflate` 5/0/5,
`decode_length` 2/0/2, `copy_distance` 3/2/1,
`build_huffman_table` 10/5/5, `decode_table_symbol` 2/0/2,
`store_dynamic_length` 2/1/1, and `decode_dynamic` 3/1/2.

Five existing claims became non-rejecting [CLM-2] redundancy advisories:
`count_slot_in_counts`, `validate_slot_in_counts`,
`offsets_slot_in_offsets`, `offsets_slot_in_counts`, and
`ordered_symbol_in_lengths`. The other 16 deflate claims remain retained; no
claim is refuted. Baseline and blinded runs retained identical obligation
counts and source order for every function.

### S10 revalidation and the boundary-path limitation

Focused checker controls now exercise all four shipped [ENT-3] S10 producers:
`read_once`, `write_once`, `host_copy_bytes`, and `host_copy_utf8`. In each
success arm the returned count bound discharges an actual indexed-access
obligation, and the existing mutation controls show that a relevant kill
invalidates it.

The real `raw_deflate_boundary.wf` path does establish `taken <= room` in the
`ReadBytes(count: taken)` arm. It does **not** currently consume that relation
in an entailment obligation: its only use of `taken` is
`filled = filled +wrap taken`. Thus the plan's earlier stronger wording that
S10 was both introduced and consumed on this real path was not satisfied by
the preregistered program. The honest evidence is “real boundary
producer, focused real-obligation consumers,” not an end-to-end boundary
consumer. Adding an otherwise unnecessary indexed access merely to make the
measurement green would be evidence-shaped program churn, so this review
candidate does not do that. On 2026-08-09 the owner accepted this honest
producer-plus-focused-consumers boundary as the S10 disposition; the milestone
then required the post-activation confirmation recorded immediately below
before it could become terminal.

## Post-activation v0.24 confirmation (2026-08-09)

Activation commit `f4c7e60` installed the exact-approved v0.24 bytes at
`spec/kernel-spec.md`, SHA-256
`53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`.
The same frozen sources, dark checker, all-claims-blinded transform, function
order, and obligation denominators were rerun against that installed authority.
The temporary in-crate probe was deleted after the run.

The installed results exactly reproduce the pre-activation candidate run, in
`total / proven / claim-supported / baseline-undischarged` form:

- utf8parse: `33 / 22 / 11 / 0`;
- SHA-256: `9 / 0 / 9 / 0`;
- deflate, full denominator: `29 / 11 / 18 / 0`; and
- deflate, dynamic-path denominator: `24 / 11 / 13 / 0`.

No previously proven site regressed. `D1h` discharges and `D1i` remains
discharged. UTF-8 retains two claims and SHA-256 retains four. Deflate retains
sixteen claims; the same five claims remain non-rejecting [CLM-2] redundancy
advisories — `count_slot_in_counts`, `validate_slot_in_counts`,
`offsets_slot_in_offsets`, `offsets_slot_in_counts`, and
`ordered_symbol_in_lengths` — and no claim is refuted. Every synthetic blinded
claim is retained.

The installed S10 confirmation also passes the focused `read_once`,
`write_once`, `host_copy_bytes`, and `host_copy_utf8` actual-index consumers
plus the kill control. The real boundary driver again establishes
`taken <= room`; it still has no natural entailment obligation that consumes
that relation. This confirms the owner-approved producer-plus-focused-consumer
boundary without upgrading it to an end-to-end boundary-consumer claim.

## Pre-activation v0.25 counted-range candidate rerun (2026-08-09)

This is review evidence, not an activation record. The same frozen-source,
dark-checker, and all-claims-blinded method was rerun against the reviewed v0.25
candidate at SHA-256
`c0b3c279f4c20d06da17ef7ac0e4ec882c8a76c560f62cce47d5b4fd4ac6beab`.
Every blinded claim became `band(p, p)` under a fresh binding; baseline and
blinded obligation order, denominators, and claim order were asserted equal.
The temporary in-crate probe was deleted after the run.

Results in `total / proven / claim-supported / baseline-undischarged` form:

- utf8parse: `33 / 22 / 11 / 0`;
- SHA-256: `9 / 9 / 0 / 0`;
- deflate, full denominator: `29 / 11 / 18 / 0`; and
- deflate, dynamic-path denominator: `24 / 11 / 13 / 0`.

Only the SHA-256 bucket moves. Its three counted index loops delete four
claims, and S11 plus the existing closure discharges all nine schedule
accesses without claim support. The compression function is `pure`, emits no
`wf_trap`, returns the direct word `3128432319_u32`, and retains the sustained
aggregate oracle and the unrelated ordinary loop. The carried-index,
next-index, missing-upper-to-length, and insufficient-lower controls remain
unproved; this result adds no general induction.

UTF-8 and all three deflate sources are byte-identical to the installed v0.24
baseline, and their aggregate and per-function buckets reproduce it exactly.
UTF-8 retains two claims. Deflate retains sixteen claims and the same five
non-rejecting redundancy advisories — `count_slot_in_counts`,
`validate_slot_in_counts`, `offsets_slot_in_offsets`,
`offsets_slot_in_counts`, and `ordered_symbol_in_lengths` — with no refuted
claim. No previously proven site regresses.

## Post-activation v0.25 confirmation (2026-08-09)

Activation commit `3e2e823` installed the exact-approved v0.25 bytes at
`spec/kernel-spec.md`, SHA-256
`c0b3c279f4c20d06da17ef7ac0e4ec882c8a76c560f62cce47d5b4fd4ac6beab`.
The same frozen sources, dark checker, all-claims-blinded transform, function
order, obligation order, claim order, and denominators were rebuilt and rerun
from that committed tree. The temporary in-crate probe was deleted after the
run, and the tracked tree returned to the activation commit with no diff.

The installed results exactly reproduce the reviewed candidate, in
`total / proven / claim-supported / baseline-undischarged` form:

- utf8parse: `33 / 22 / 11 / 0`;
- SHA-256: `9 / 9 / 0 / 0`;
- deflate, full denominator: `29 / 11 / 18 / 0`; and
- deflate, dynamic-path denominator: `24 / 11 / 13 / 0`.

UTF-8 and every deflate per-function bucket remain identical to the installed
v0.24 baseline, so no previously proven site regresses. Deflate retains
sixteen claims; the same five claims remain non-rejecting [CLM-2] redundancy
advisories — `count_slot_in_counts`, `validate_slot_in_counts`,
`offsets_slot_in_offsets`, `offsets_slot_in_counts`, and
`ordered_symbol_in_lengths` — and no claim is refuted. Every synthetic blinded
claim is retained, and every baseline obligation is discharged.

The installed SHA source contains exactly three counted ranges, retains the
unrelated ordinary `loop @sustained_hashing`, and contains no claim. All nine
schedule subscripts remain discharged with claims blinded. The backend and
runtime controls independently confirm that the worker is `pure`, contains no
`wf_trap`, preserves the rotate and indexed-address shapes, returns the direct
word `3128432319_u32`, and retains the sustained aggregate oracle. Empty,
reversed, singleton, maximum-edge, captured-endpoint, and nested-break counted
ranges execute without a hidden trap. The carried-index, next-index,
missing-upper-to-length, and insufficient-lower controls remain unproved; the
installed result still adds no general induction.

## Pre-activation v0.26 requires-goal candidate rerun (2026-08-09)

This is review evidence, not an activation record. The same frozen-source,
dark-checker, and all-claims-blinded method was rerun against the reviewed
v0.26 candidate at SHA-256
`18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`.
Baseline and blinded runs asserted the same function order, obligation count,
claim count, and claim order; every synthetic blinded claim was retained. The
temporary in-crate probe was then deleted, and its host file returned to its
exact pre-probe SHA-256.

Results in `total / proven / claim-supported / baseline-undischarged` form:

- utf8parse: `33 / 22 / 11 / 0`;
- SHA-256: `9 / 9 / 0 / 0`;
- deflate, full denominator: `29 / 11 / 18 / 0`; and
- deflate, dynamic-path denominator: `24 / 11 / 13 / 0`.

Every bucket and every deflate per-function vector is identical to installed
v0.25: `read_bits` 1/1/0, `emit_byte` 1/1/0, `inflate` 5/0/5,
`decode_length` 2/0/2, `copy_distance` 3/2/1,
`build_huffman_table` 10/5/5, `decode_table_symbol` 2/0/2,
`store_dynamic_length` 2/1/1, and `decode_dynamic` 3/1/2. UTF-8 still splits
as `parse` 11/0/11 and `main` 22/22/0. SHA-256 remains 9/9 in
`sha256_abc_word_zero` with no claim.

Deflate retains sixteen claims and the same five non-rejecting redundancy
advisories — `count_slot_in_counts`, `validate_slot_in_counts`,
`offsets_slot_in_offsets`, `offsets_slot_in_counts`, and
`ordered_symbol_in_lengths` — with no refuted claim. The requires migration
changes exact effect rows where the old executable prologue was their only
read contributor, but changes no indexed computation or claim in the frozen
UTF-8/deflate bodies. The three `decode_dynamic` calls to
`store_dynamic_length` independently pass the new caller-side requirement
judgment without S2/S3 evidence, while the callee retains the same S4 body
axiom and the `distance_position_in_lengths` claim remains present. Thus the
new call boundary introduces no regression in the frozen obligation buckets
and does not disguise one with a retained callee-entry check.

## Post-activation v0.26 confirmation (2026-08-10)

Activation commit `441cd5b833096d558549bb09aeecfcfe63340584` installed
the exact-approved v0.26 bytes at `spec/kernel-spec.md`, SHA-256
`18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`,
and archived the byte-identical outgoing v0.25 bytes. The same frozen sources,
dark checker, all-claims-blinded transform, function order, obligation order,
claim order, and denominators were rebuilt and rerun from that committed tree.
The temporary in-crate probe was then deleted with its host files restored to
their pre-probe hashes; the tracked tree and index returned clean at the same
activation commit.

The installed results exactly reproduce the reviewed candidate, in
`total / proven / claim-supported / baseline-undischarged` form:

- utf8parse: `33 / 22 / 11 / 0`;
- SHA-256: `9 / 9 / 0 / 0`;
- deflate, full denominator: `29 / 11 / 18 / 0`; and
- deflate, dynamic-path denominator: `24 / 11 / 13 / 0`.

Every synthetic blinded claim remains retained and every baseline obligation
is discharged. Deflate retains sixteen claims; the same five claims remain
non-rejecting [CLM-2] redundancy advisories — `count_slot_in_counts`,
`validate_slot_in_counts`, `offsets_slot_in_offsets`,
`offsets_slot_in_counts`, and `ordered_symbol_in_lengths` — and no claim is
refuted. All three real `store_dynamic_length` calls remain discharged in both
the unasserted and S4-blinded rewalks, while the distance claim remains present.

Independent installed controls confirm that ordinary required functions emit
no executable callee prologue, real process-entry checks retain their exact
failure behavior, counted-range maximum-edge execution has no hidden trap, and
the SHA-256 worker retains its no-`wf_trap` shape and sustained runtime oracle.
The complete repository gate is green. The separately invoked adapter reports
`Pass=393 Fail=1 Skip=13`; its sole failure remains the pre-existing OWN-3
`RegionsAndBorrows` unsupported boundary.

## Pre-activation v0.27 provenance-gate candidate rerun (2026-08-10)

This is candidate review evidence, not an installed-language or activation
record. The rerun used the prepared v0.27 bytes at `spec/kernel-spec.md`,
SHA-256
`bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`,
and the byte-identical prepared outgoing-v0.26 archive at
`spec/kernel-spec-v0.26.md`, SHA-256
`18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`.
At that review checkpoint both remained non-authoritative and uncommitted
pending the required explanation, hard wait, exact owner approval, and atomic
activation.

The same frozen-source dark-checker and all-claims-blinded method reports, in
`total / proven / claim-supported / baseline-undischarged` form:

- utf8parse: `33 / 22 / 11 / 0`;
- SHA-256: `9 / 9 / 0 / 0`;
- deflate, full denominator: `29 / 24 / 5 / 0`; and
- deflate, dynamic-path denominator: `24 / 19 / 5 / 0`.

The frozen four-source boundary-fed unit now contains twelve claim
declarations. Seven remain load-bearing and five are non-rejecting CLM-2
redundancy advisories; none is refuted:

- retained: `copy_read_in_source`, `copy_write_in_destination`,
  `count_symbol_in_lengths`, `order_symbol_in_lengths`,
  `walk_length_in_counts`, `code_index_in_order`, and
  `end_symbol_in_literals`;
- redundant: `count_slot_in_counts`, `validate_slot_in_counts`,
  `offsets_slot_in_offsets`, `offsets_slot_in_counts`, and
  `ordered_symbol_in_lengths`.

Exactly thirteen formerly claim-supported protected sites now discharge through
real branches and the existing domain error values. Their function breakdown
is: `inflate` 5 (the four stored-header reads and stored-copy read),
`decode_length` 2 (the two length-table reads), `copy_distance` 1 (the history
read), `build_huffman_table` 3 (the offset read, offset write, and symbol
write), `decode_table_symbol` 1 (the ordered-symbol read), and
`store_dynamic_length` 1 (the distance-length write). The selected outcomes
are five `Truncated`, one `InvalidHuffmanCode`, one `InvalidDistance`, and four
`InvalidHuffmanTree` branch repairs. The thirteen-site count exceeds the eleven
removed claims because `length_symbol_in_tables` and `order_slot_in_offsets`
each supported two indexed accesses.

The PRV gate's S2/S3-off rewalk covers the same obligation identities, while
the complete-state outcomes remain the base acceptance judgment. All thirteen
repaired sites remain discharged with S2 and S3 absent because their real
branches establish the relation. The five remaining claim-supported sites
have internal subjects, so PRV-3 leaves their ordinary full-state S3
authorization intact; this is not a claim that those five become
proof-independent. The three
`store_dynamic_length` calls also establish the exact instantiated requirement
in the complete, unasserted, and S4-blinded rewalks. Its distance write now has
its own `InvalidHuffmanTree` value branch, so neither the callee S4 axiom nor an
entry wrapper supplies provenance authorization.

The frozen raw-DEFLATE program test reports 3/3. Stored, fixed, and dynamic
success vectors; boundary, truncated, malformed, and oversize errors; closed
output; semantic checking; LLVM lowering; cleanup; and facts-on/facts-off
runtime behavior retain their existing oracles. The only exact effect-row
change is removal of the now-unexhibited `traps` category from
`store_dynamic_length`, `decode_length`, `copy_distance`, and `decode_fixed`;
all other effect categories and rows remain unchanged.

The complete native adapter over the unchanged 407 prior identities plus
sixteen additive PRV cases reports `Pass=409 Fail=1 Skip=13`. The sole failure
remains the pre-existing OWN-3 `RegionsAndBorrows` unsupported boundary; the
additive cases all pass and do not hide that retained result.

## Post-activation v0.27 confirmation (2026-08-10)

Activation commit `5ab45aa73a1a713e994773d2c04c34400795950a` installed
the exact-approved v0.27 bytes at `spec/kernel-spec.md`, SHA-256
`bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`,
and archived the byte-identical outgoing v0.26 bytes at
`spec/kernel-spec-v0.26.md`, SHA-256
`18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`.
The installed dark-checker probe used the same all-claims-blinded transform,
function order, obligation order, claim order, and denominators as candidate
review. Its frozen source identities were
`55ae93bae65216e495a0dc4f87ef245b25c8731a807f8b27d85a244b4c0095e1`
for `utf8parse.wf`,
`a0a493bb6dd23c542a22e0fee775a5eabedc71d10fd91084cf90637447de03e1`
for `sha256_abc.wf`, and, in the frozen four-source compilation order,
`raw_deflate.wf`
`5e87885a519de539736b0ad6a619a2c92bdf659623e3f39ded31116e63adb585`,
`raw_deflate_dynamic.wf`
`2606ae0b81039b0ecc787df2fa9cb87279ba44663744540d48f6abeea984d4c5`,
`raw_deflate_dynamic_decode.wf`
`72129284c60a6eacbe2bb86d7d3a82375ed2270ea457c49d8e1db95064fc960f`,
and `raw_deflate_boundary.wf`
`3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4`.
The installed conformance manifest identity is
`04d2562f41eecbd3af5770c96ccad9a4fcfa8cd9f9d849c414f1cccbb89d072d`.

The installed results exactly reproduce the reviewed candidate, in
`total / proven / claim-supported / baseline-undischarged` form:

- utf8parse: `33 / 22 / 11 / 0`;
- SHA-256: `9 / 9 / 0 / 0`;
- deflate, full denominator: `29 / 24 / 5 / 0`; and
- deflate, dynamic-path denominator: `24 / 19 / 5 / 0`.

The boundary-fed unit retains twelve claims: seven are load-bearing, five are
non-rejecting CLM-2 redundancy advisories, and none is refuted. All thirteen
formerly claim-supported sites remain authorized by their real value branches
with S2 and S3 absent. The three `store_dynamic_length` callers continue to
prove the exact instantiated requirement, including in the unasserted and
S4-blinded rewalks; no retained prologue or hidden assertion supplies the
protected-leaf authorization.

The focused provenance suite passes 41/41 and the frozen raw-DEFLATE runtime
oracle passes 3/3. The separately invoked native adapter reports
`Pass=409 Fail=1 Skip=13`; its sole failure remains the pre-existing OWN-3
`RegionsAndBorrows` unsupported boundary. The complete `make check` gate is
green with 698/698 library tests, 30/30 real-program tests, 131/131 rule
coverage, and all 19 activation-chain entries verified. Commit
`7451230944524b03f6b95900b46e129e9dab809e` records the installed bounded
provenance decision in the live design memory, whose lint also passes.
The temporary in-crate probe and its host-module declaration were removed,
their exact pre-probe bytes were restored, the detached activation worktree
and scratch artifacts were deleted, and both the activation tree before
removal and the shared tree after the run had clean worktree and index state.
