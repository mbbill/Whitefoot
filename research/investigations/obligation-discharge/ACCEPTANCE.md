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

## Stage 8a bit-bound proof refresh after DIAG-2 root retention (2026-08-13)

This is observational proof-feasibility evidence, not installed semantics. The
complete removable probe was recreated and rerun at exact execution revision
`36410174dfac97d76b6f30cf26e8bfd0c10aab5a`, after task 0055 implementation
`491446af053bfe8db95941e6093b30f4ff9cfb7a` and closure
`a94ddd8a4bdaabf0a4e739c6409cc09402e60790`. The former 0051 section, whose
SHA-256 is
`8783689f2dd8b7f9da5d857d9d673075625c472fcbca05a1c0dfb52f13995a99`,
and the withdrawn combined candidate ending in
`78ce0073244e810c1acb1b094c86d58d0522800ce025fc1f197c369fb84d53d5`
were not reused as current evidence.

The active v0.27 specification identity was
`bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`.
The frozen real sources matched the Current Plan before and after the probe:

- `raw_deflate.wf`:
  `5e87885a519de539736b0ad6a619a2c92bdf659623e3f39ded31116e63adb585`;
- `raw_deflate_dynamic.wf`:
  `2606ae0b81039b0ecc787df2fa9cb87279ba44663744540d48f6abeea984d4c5`;
- `raw_deflate_dynamic_decode.wf`:
  `72129284c60a6eacbe2bb86d7d3a82375ed2270ea457c49d8e1db95064fc960f`;
- `raw_deflate_boundary.wf`:
  `3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4`;
  and
- `wfgrep.wf`:
  `a1e49bcb9ffd353e707d4bbafe1eb4a2b634b9177c6916b2ff7b503ec5dff0bd`.

The unmodified pre-probe command was:

```text
env TMPDIR=/Users/bytedance/do_not_scan make -C compiler check
```

Its first sandboxed invocation stopped before a test because Cargo could not
create its target lock under the isolated worktree. The identical approved
retry is the measurement: it ended `WHITEFOOT COMPILER GATE GREEN`, with
718/718 compiler-library tests in 560.36 seconds, 30/30 real-program tests in
1,857.98 seconds, warning-free formatting, clippy and documentation, and the
131-rule specification plus all 19 activation-chain entries intact.

Before either temporary source existed, a scratch clone of the real
`read_bits` body called a pure helper requiring `value < mask_high`
immediately before its sole normal `Ok(value:)` return. Its early
`Err(Truncated())` return had no helper call. This exact command selected one
test:

```text
env TMPDIR=/Users/bytedance/do_not_scan cargo test --manifest-path compiler/Cargo.toml --locked --offline stage8a_real_read_bits_mask_bound_is_unproved_without_bit_sources -- --nocapture
```

It passed 1/1: the single retained call goal was `Unproved`, with empty
evidence and no derivation root, and the refreshed derivation validator
accepted the summary.

The probe then temporarily added exactly two general checked-tree sources to
`compiler/src/semantic/entailment/flow/sources.rs`:

1. unsigned `iand(left, right)` established its result at most each operand
   that was an admitted term or checked constant; and
2. unsigned `ishl.wrap(one, count)` established its result distinct from zero
   only when the value operand was directly the checked literal or named
   constant whose mathematical value is one.

Both used the existing term table, fact state, closure, S7 event kind,
support, kill and call-goal judgment. No function, source, project, corpus or
test identity participated. No third source, arithmetic-expression term,
Boolean decomposition, induction, fixed point or solver was added. The
temporary host identities were:

| Temporary file | SHA-256 | Lines | Bytes |
| --- | --- | ---: | ---: |
| `compiler/src/semantic/entailment/flow/sources.rs` | `62e3296d701acf08ef2847d767533951519fddfe3bcae67cdabdcf5fc649439c` | 977 | 36,350 |
| `compiler/src/semantic/tests/entailment.rs` | `e885f072fc7d358d05eb4fece0a5185804d7e9510c63bc21d9faa6da1b383a96` | 6,769 | 206,556 |

The positive matrix retained exactly 192 call goals: four unsigned widths,
six counts (`0`, `1`, `W-2`, `W-1`, `W`, `W+1`), literal and named-constant
one, and four `iand` term/constant and operand-order forms. A first fixture put
all 192 cases in one function. Its entailment closure remained in the first
analysis after more than an hour, so the lead approved terminating only that
test process from this exact aggregate command:

```text
env TMPDIR=/Users/bytedance/do_not_scan cargo test --manifest-path compiler/Cargo.toml --locked --offline --lib stage8a_ -- --test-threads=1
```

The approved termination exited 143 before validation or any result
assertion. That run is nonterminal, superseded fixture evidence: it is neither
a pass nor a failure and contributes no positive count.

The equal-strength replacement used 24 functions, one for each `(width,
count-slot)` pair and eight cases per function. One `with_semantics_dark`
analysis collected the 24 explicitly named summaries from one compiled
program; it did not recompile the program per summary. Every summary passed
`validate_derivations`, retained exactly eight goals, and every goal was
`Discharged` with only `ExactL0Projection`. A second whole-program analysis
validated all summaries again and its complete ordered summary vector was
exactly equal to the first. This command exited 0 and passed 1/1 in 160.11
seconds:

```text
env TMPDIR=/Users/bytedance/do_not_scan cargo test --manifest-path compiler/Cargo.toml --locked --offline stage8a_unsigned_bit_sources_discharge_192_cases_deterministically -- --nocapture
```

The refreshed exact root for each mask-bound query is a four-node chain. In
normalized difference-bound form it is:

```text
SourceBound(mask - high <= -1, S7)
SourceBound(value - mask <= 0, S7)
TransitiveBound(value - high <= -1)
GoalProjection(value < high, ExactL0Projection)
```

An initial grouped oracle incorrectly required the temporary
`SourceDistinct(high, ZERO)` node itself to be an ancestor of that final
query root. The same grouped command shown above exited 101 with 0/1 passed
after 81.59 seconds at the first otherwise discharged goal. Read-only review
established that this was an oracle error, not a disposition change or a
DIAG-2 dangling-parent defect. The distinct fact is live when the existing
wrapping-subtraction source decides that
`mask = high -wrap 1` is in range. Once that S7 relation is admitted, it is a
zero-parent `SourceBound`; the later query starts from that established fact,
and derivation finalization correctly prunes the eligibility-only
`SourceDistinct` node. The corrected oracle therefore checks the exact two S7
`SourceBound` parents, their transitive relation and the goal projection,
rather than claiming that eligibility evidence is a retained query parent.

A paired counterfactual separately proves that the shift source is still
necessary. Both functions retained the unsigned `iand` source and the same
mask-bound goal. The positive function passed literal `1_u64` directly to
`ishl.wrap`; the other first bound the same value to a local `local_one` and
therefore did not satisfy the deliberately bounded checked-constant source.
The direct form was `Discharged` with the exact root above. The local-binding
form was `Unproved`, with no positive evidence or derivation root. Both
summaries were recomputed and compared exactly. This command exited 0 and
passed 1/1 in 0.08 seconds:

```text
env TMPDIR=/Users/bytedance/do_not_scan cargo test --manifest-path compiler/Cargo.toml --locked --offline stage8a_shift_source_is_required_for_the_combined_mask_bound -- --nocapture
```

The remaining focused matrices retained these exact outcomes:

| Class | Construction | Result |
| --- | --- | --- |
| positive | 24 grouped functions x 8 cases | 192 discharged, 0 unproved, 0 refuted |
| near miss | four signed `iand`, four signed shifts, `ior`, `ixor`, right shift, trapping shift, zero/two/nonconstant shift value, one in the wrong position, and the two each-source-alone controls | 0 discharged, 18 unproved, 0 refuted |
| support and kill | 11 functions and 22 call goals | 15 discharged, 7 unproved, 0 refuted |
| real `read_bits` normal path | helper after the real `state.hold` and `state.bits` writes | 1 discharged, 0 unproved, 0 refuted |
| paired source counterfactual | direct constant-one versus local-binding-one | 1 discharged, 1 unproved, 0 refuted |

The signed and operation near-miss command exited 0 and passed 1/1:

```text
env TMPDIR=/Users/bytedance/do_not_scan cargo test --manifest-path compiler/Cargo.toml --locked --offline stage8a_signed_and_operation_near_misses_keep_18_goals_unproved -- --nocapture
```

Every one of its 18 goals was `Unproved`, with empty evidence and no root,
including the separate `iand`-only and shift-only cases. Its complete summary
repeated exactly. The real-body positive command also exited 0 and passed 1/1:

```text
env TMPDIR=/Users/bytedance/do_not_scan cargo test --manifest-path compiler/Cargo.toml --locked --offline stage8a_real_read_bits_mask_bound_is_discharged_with_bit_sources -- --nocapture
```

It retained one `Discharged` call goal, only `ExactL0Projection`, the exact
two-source-bound root above, and an exactly equal repeated summary. Because
the helper occurs only before the sole normal return, the `Err` edge publishes
no normal-result witness.

The support/kill command exited 0 and passed 1/1:

```text
env TMPDIR=/Users/bytedance/do_not_scan cargo test --manifest-path compiler/Cargo.toml --locked --offline stage8a_bit_source_support_and_kill_matrix_is_exact -- --nocapture
```

Its two booleans are respectively the `value <= iand operand` goal and the
combined `value < high` goal:

| Function | Discharged goals |
| --- | --- |
| base | `true, true` |
| sibling scalar write | `true, true` |
| sibling element write | `true, true` |
| projected whole-root write | `false, true` |
| real `state.hold` write | `false, true` |
| mask write | `true, false` |
| value write | `false, false` |
| high write | `true, false` |
| reversed operand order | `true, true` |
| constant operand | `true, true` |
| scope delivery | `true, false` |

Every discharged support goal retained only `ExactL0Projection`, a valid
root, and an S7 source; every unproved goal retained empty evidence and no
root. Each function summary repeated exactly. A fixture-only projected-array
write initially reached the unrelated `RegionsAndBorrows` unsupported path;
the final sibling-element control uses a local array element and changes no
compiler behavior. No result from the unsupported fixture is counted.

These results isolate the real surviving path:

```text
high != 0                             temporary unsigned shift source
mask = high - 1                       existing wrapping-offset S7 eligibility
value <= mask                         temporary unsigned iand source
value <= mask < high                  existing ENT-4 closure
```

The later `state.hold` write kills the separately supported
`value <= old state.hold` relation, but names no support in the mask route.
The new DIAG-2 retained ledgers changed none of the intended positive,
negative, support/kill or real-body dispositions. Every final positive and
negative summary passed `validate_derivations`; no retained term, event,
parent or root ID dangled.

Every temporary compiler and test edit was then removed. The restored files
matched the exact pre-probe identities and sizes:

| Restored file | SHA-256 | Lines | Bytes |
| --- | --- | ---: | ---: |
| `compiler/src/semantic/entailment/flow/sources.rs` | `73ddc0beb6aca9ff1443ec502a9745bc2d6dd4c5e565b73628e74a43e0197abd` | 895 | 33,505 |
| `compiler/src/semantic/tests/entailment.rs` | `94a6fd9579d163fd5ee9b72aa41f5d6550db26c664fb05041c7768aa74a05e0c` | 6,142 | 185,244 |
| `compiler/src/semantic/tests.rs` | `afd0784147c3b807e9593226ebd8542db5e2a6bd7fe34adccd84e74ca061c418` | 1,149 | 38,492 |

The restored focused command passed 112/112 tests in 340.23 seconds, with 606
filtered:

```text
env TMPDIR=/Users/bytedance/do_not_scan cargo test --manifest-path compiler/Cargo.toml --locked --offline --lib semantic::tests::entailment -- --test-threads=1
```

The persistent result tree was then accepted by these two sequential commands:

```text
env TMPDIR=/Users/bytedance/do_not_scan make -C compiler check
env TMPDIR=/Users/bytedance/do_not_scan make check
```

The post-restoration compiler gate ended `WHITEFOOT COMPILER GATE GREEN`:
718/718 library tests passed in 486.16 seconds, all 30/30 real-program tests
passed in 1,215.98 seconds, including wfgrep 9/9 and raw-DEFLATE 3/3, and
formatting, clippy, documentation, the 131-rule specification and all 19
activation records were green. The repository gate first verified all 28
recorded specification archives, passed 23/23 independent conformance-runner
tests, and reported 131/131 rule coverage. Its nested compiler gate then
passed 718/718 library tests in 518.53 seconds and 30/30 real programs in
1,202.69 seconds before ending both `WHITEFOOT COMPILER GATE GREEN` and
`WHITEFOOT GATE GREEN (active compiler + independent evidence)`.

The isolated repository remained clean at exact revision
`36410174dfac97d76b6f30cf26e8bfd0c10aab5a`. The installed acceptance record
remained
`271abdf48dcb71e7698f8f1e1d5c18c23adf256115278e5f7ec7ca25226d7df3`,
and `AGENTS.md` and `CLAUDE.md` remained byte-identical at
`f5124595cc56256ec7ac6bf10b63b01bec645410db110849ade3f809210109ee`.
No persistent compiler, real-program, specification, conformance, generated,
design-memory, task-record, approval-ledger or installed-acceptance byte was
changed.

This result establishes only that the two bounded local sources suffice for
the measured normal-result path. It does not install either source, transfer a
fact through the fourteen callers' outer bindings, define user-function
postconditions, or authorize Stage 8b. This section is ordinary research
evidence under the active plan. With tasks 0051 and 0052 terminal-positive,
task 0053 may now enumerate the complete caller map; Stage 8b remains gated on
that caller result, the independent DIAG-2 trust prerequisite, and its own
separate specification and protected-conformance approval.

## Stage 8a counted-append proof refresh after DIAG-2 root retention (2026-08-13)

The complete corrected probe was recreated and rerun at exact execution
revision `36410174dfac97d76b6f30cf26e8bfd0c10aab5a`, after task 0055
implementation `491446af053bfe8db95941e6093b30f4ff9cfb7a` and closure
`a94ddd8a4bdaabf0a4e739c6409cc09402e60790`. The former section and combined
candidate ending in `78ce0073244e810c1acb1b094c86d58d0522800ce025fc1f197c369fb84d53d5`
were not reused as current evidence. This run changed no persistent compiler,
program, specification, conformance, generated, design-memory, task-record, or
installed-acceptance byte.

The pre-probe command

```text
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp make -C compiler check
```

passed at that revision: 718/718 compiler library tests and 30/30 real-program
tests passed, the documentation build was warning-free, and the specification
gate verified 131 rules and all 19 activation-chain entries before ending
`WHITEFOOT COMPILER GATE GREEN`.

Before any counted candidate was compiled, the unmodified ordinary-loop body
could not prove its instantiated `result <= capacity` helper requirement. The
compiler reported FN-8 `UndischargedCallRequirement` with disposition
`Unproved` at call-node path `[1,0,9,0,0]`. The separately compiled exact
invalid-domain witness then reproduced the current body's behavior for
`capacity=3, filled=4, len(text)=0`: result `4`, with all three destination
bytes still `0xA5`.

The counted candidate used the existing
`for @append at in filled..capacity` form and this exact current-language
requirement:

```whitefoot
requires {
  let capacity = len(deref(destination));
  let admitted = ile(filled, capacity);
  check admitted else trap "append filled exceeds destination";
}
```

Its body computed `taken = at -wrap filled`, returned `at` on the true `done`
edge, otherwise read `text[taken]` and wrote `destination[at]`, then returned
`capacity` after exhaustion. The combined source compiled, and independent
early-return-only and exhaustion-only helper sources each compiled. Thus both
normal return shapes still prove the same result bound after task 0055. The
false `done` edge proved the text index and S11 proved the destination index;
the probe introduced no induction, loop fixed point, subtraction relation,
post-loop binder equality, claim, or fallback check to prove the result.

The corrected proof and behavioral controls retained these exact outcomes.
The pair ordering below is current ordinary body versus counted variant:

| Control | Result-bound proof | Behavioral witness |
| --- | --- | --- |
| Remove the requirement | proved | invalid domain: `4 vs 3` |
| Early `return at +wrap 1` | proved | admitted empty text: `0 vs 1` |
| Early `return at +wrap 2` | FN-8 `Unproved` | proof-free admitted control: `0 vs 2` |
| Restore the ordinary loop | FN-8 `Unproved` | not applicable |
| Return an independent parameter | FN-8 `Unproved` | not applicable |
| Consume the binder after the loop | TYPE-5 `InvisibleUse` | not applicable |

One executable asserted all three behavioral pairs and that both three-byte
`0xA5` destinations remained unchanged in each pair; it exited zero. The
first two controls therefore preserve their mathematically valid local bound
proofs while falsifying behavioral equivalence. The `at +wrap 2` result is the
arithmetic proof-negative control. No negative control gained a hidden premise
or fallback.

The final advisory-free differential executable checked exactly
`(1 + ... + 9) * 9 * 2 * 3 = 2,430` cases: capacities and text lengths
`0..=8`, every `filled` in `0..=capacity`, destination fills `0x00` and
`0xA5`, and all-zero, all-`0xFF`, and ascending text. It asserted the exact
case count, return equality, and equality of every destination byte, then
exited zero. All 2,430 admitted cases matched. The newly retained complete
DIAG-2 counted-root evidence changed none of the required positive, negative,
hostile-control, or behavioral outcomes.

Every final temporary source was compiled through the local
`compiler/target/debug/whitefootc`; executable witnesses used `-o` and were
run directly. Their exact UTF-8 identities and byte/line counts were:

| Source | SHA-256 | Lines | Bytes |
| --- | --- | ---: | ---: |
| ordinary-loop result-bound negative | `04add72fcaa5065f985b9a8a51c5d941918772c2b97c05b0820a83266dffead5` | 34 | 881 |
| invalid-domain current-body witness | `e8aebd622b0bd9e2f5b95fb50e00e4d619d2dfbe729bc4033b00dad6fd053920` | 38 | 1,386 |
| combined counted proof | `d62d2c7ad45c3e2a31b1449579782246834d5138780ae4fde63cea3007d56f18` | 31 | 964 |
| early-return-only proof | `28c6d7f6af4b2d1e5884d22228e53a0cc75af6322389afc4f3a74f9dde98a926` | 30 | 914 |
| exhaustion-only proof | `3727624f0f4ccabf93827f01ee2acb6f0d41ede572311fabec2bee98fe96265c` | 30 | 921 |
| no-requirement proof control | `ca6911aec64d6818ccd00c9faf330bd540f7a1f1a67682b14343bed6898632a0` | 27 | 817 |
| `at +wrap 1` proof control | `79472bb2b2ffabe40f4109ad37f2dfe8a22d32392523e000fa81092798956219` | 32 | 1,008 |
| `at +wrap 2` proof-negative control | `a7d691b5dda35c9baded39fcebbdb7fc6a488a2217664b4fd2e3ac756fe657d3` | 31 | 952 |
| independent-parameter control | `ec1423884d84313f25690780d09542d6dfbf0815020c04ca6cc66be27b63def7` | 30 | 946 |
| post-loop-binder control | `0c753a4a6d8e40ec7e686c3435eaf6fb95692b4f901294091e45a8072c7332ce` | 22 | 653 |
| combined behavioral controls | `b08221fa91ea3a658aa12c4bdfba38608d540c6c538b3376b01ebbe1245530c8` | 151 | 7,921 |
| 2,430-case admitted differential | `4abea03fb753a76e6777050bec7ae5bf5da008a17e47bb121ab9a953b870756f` | 154 | 6,011 |

The original execution record retained the exact final compiler and executable
argv and their direct statuses. The commands below are the final commands for
the hash-identified inputs above, not earlier source-construction attempts:

```text
compiler/target/debug/whitefootc --emit-llvm /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/ordinary-result-bound-negative.wf
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp compiler/target/debug/whitefootc -o /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/current-invalid-domain /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/current-invalid-domain.wf
/Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/current-invalid-domain
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp compiler/target/debug/whitefootc -o /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/counted-combined-proof /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/counted-combined-proof.wf
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp compiler/target/debug/whitefootc -o /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/counted-early-proof /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/counted-early-proof.wf
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp compiler/target/debug/whitefootc -o /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/counted-exhaustion-proof /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/counted-exhaustion-proof.wf
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp compiler/target/debug/whitefootc -o /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/control-no-requirement-proof /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/control-no-requirement-proof.wf
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp compiler/target/debug/whitefootc -o /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/control-plus-one-proof /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/control-plus-one-proof.wf
compiler/target/debug/whitefootc --emit-llvm /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/control-plus-two-negative.wf
compiler/target/debug/whitefootc --emit-llvm /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/control-independent-negative.wf
compiler/target/debug/whitefootc --emit-llvm /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/control-post-loop-binder-negative.wf
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp compiler/target/debug/whitefootc -o /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/behavior-controls /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/behavior-controls.wf
/Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/behavior-controls
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp compiler/target/debug/whitefootc -o /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/admitted-differential-2430 /Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/admitted-differential-2430.wf
/Users/bytedance/do_not_scan/whitefoot-0052-post0055-probes/admitted-differential-2430
```

The ordinary-loop command exited `1` with FN-8 `Unproved` at
`[1,0,9,0,0]`. The invalid-domain compile and execution, the combined proof,
both return-only proofs, the no-requirement proof, and the `at +wrap 1` proof
each exited `0`. The `at +wrap 2` and independent-parameter commands each
exited `1` with FN-8 `Unproved`, at `[1,0,7,0,4,0,2,0,0]` and
`[1,0,7,0,4,0,1,0,0]` respectively. The post-loop-binder command exited `1`
with TYPE-5 `InvisibleUse` at `[0,0,8,0,0,0,0,0]`. The final behavioral and
differential compiles and executions each exited `0`. Expected source
rejections are therefore distinguished from successful executable witnesses;
no shell pipeline obscured any status.

The unchanged real-program oracles were replayed exactly with:

```text
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp cargo test --manifest-path compiler/Cargo.toml --test programs programs::wfgrep:: --locked --offline -- --nocapture
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp cargo test --manifest-path compiler/Cargo.toml --test programs programs::raw_deflate:: --locked --offline -- --nocapture
```

They passed wfgrep 9/9 and raw-DEFLATE 3/3. The frozen source identities were:

- `wfgrep.wf`:
  `a1e49bcb9ffd353e707d4bbafe1eb4a2b634b9177c6916b2ff7b503ec5dff0bd`;
- `raw_deflate.wf`:
  `5e87885a519de539736b0ad6a619a2c92bdf659623e3f39ded31116e63adb585`;
- `raw_deflate_dynamic.wf`:
  `2606ae0b81039b0ecc787df2fa9cb87279ba44663744540d48f6abeea984d4c5`;
- `raw_deflate_dynamic_decode.wf`:
  `72129284c60a6eacbe2bb86d7d3a82375ed2270ea457c49d8e1db95064fc960f`;
  and
- `raw_deflate_boundary.wf`:
  `3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4`.

After capture of the temporary source identities, every scratch source and
executable and the scratch compiler temporary directory were removed. The
isolated repository remained at exact execution revision
`36410174dfac97d76b6f30cf26e8bfd0c10aab5a` with a clean index and worktree.
The active specification remained
`bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`;
the installed acceptance record remained
`271abdf48dcb71e7698f8f1e1d5c18c23adf256115278e5f7ec7ca25226d7df3`;
and `AGENTS.md` and `CLAUDE.md` remained byte-identical at
`f5124595cc56256ec7ac6bf10b63b01bec645410db110849ade3f809210109ee`.

After restoration, these commands ran serially in the clean isolated worktree:

```text
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp make -C compiler check
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0052-post0055-cargo-tmp make check
```

The compiler gate exited `0`. Its compiler library suite passed 718/718, its
real-program suite passed 30/30, documentation built with warnings denied, and
the specification gate verified 131 rules and 19 unbroken activations before
ending `WHITEFOOT COMPILER GATE GREEN`. The repository gate then exited `0`:
the append-only check reported no released-specification change, all 28
recorded specifications hashed as recorded, conformance structure passed
23/23, and coverage was 131/131 rules with zero uncovered. Its nested compiler
gate again passed 718/718 library and 30/30 real-program tests, built the
documentation with warnings denied, and verified the same 131 rules and 19
activations before the repository command ended
`WHITEFOOT GATE GREEN (active compiler + independent evidence)`. As the root
Makefile states, that repository gate checks conformance structure and declared
coverage rather than every case verdict; this task changed no conformance or
canonical-runner byte.

The task-owned gate temporary directory was then removed. A final check found
both task scratch directories absent, the index and worktree clean at exact
revision `36410174dfac97d76b6f30cf26e8bfd0c10aab5a`, and every specification,
installed-acceptance, instruction, and real-source identity stated above
unchanged. This refreshed section is ordinary research evidence local to the
stated admitted domain. It does not claim or authorize a language,
conformance, canonical-gate, or compiler-acceptance change. With tasks 0051
and 0052 terminal-positive, task 0053 may now enumerate the complete caller
map; Stage 8b remains subject to its separate specification and
protected-conformance approval.

## DIAG-2 exact-derivation retention and bounded-cost confirmation (2026-08-13)

Task 0056 records **PASS** evidence for the bounded existing-DIAG-2 repair;
this ordinary research result closes the trust measurement when its section
and lifecycle record land through normal lead review. The audit compares exact
task-0055 terminal closure
`a94ddd8a4bdaabf0a4e739c6409cc09402e60790` with plan-activation baseline
`c2c40924b5b7a4ac4fbcb54a3b88b9d025285e7d`. The checked program retains the
complete required derivation set through one shared function-local DAG, its
ledger-owned storage is `O(S + P + R + C)`, and every frozen source remains
accepted with the same diagnostics and runtime oracles. This records
already-active DIAG-2 behavior; it changes no language semantics, source
acceptance, runtime behavior, lowering authority, or required check.

The corrected task-local storage definition is fixed by task-contract revision
`48a92a94a3d9cf49dc5ff998741eddd3f8a8aea2`. The active specification stayed
byte-identical at
`bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`.
The measurement-time acceptance record stayed byte-identical at
`271abdf48dcb71e7698f8f1e1d5c18c23adf256115278e5f7ec7ca25226d7df3`.
Baseline and candidate had identical frozen source hashes:
`utf8parse.wf`
`55ae93bae65216e495a0dc4f87ef245b25c8731a807f8b27d85a244b4c0095e1`,
`sha256_abc.wf`
`a0a493bb6dd23c542a22e0fee775a5eabedc71d10fd91084cf90637447de03e1`,
the ordered DEFLATE files
`5e87885a519de539736b0ad6a619a2c92bdf659623e3f39ded31116e63adb585`,
`2606ae0b81039b0ecc787df2fa9cb87279ba44663744540d48f6abeea984d4c5`,
`72129284c60a6eacbe2bb86d7d3a82375ed2270ea457c49d8e1db95064fc960f`,
and `3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4`,
and `wfgrep.wf`
`a1e49bcb9ffd353e707d4bbafe1eb4a2b634b9177c6916b2ff7b503ec5dff0bd`.

### Complete mandatory roots and retained representation

A temporary in-crate audit used the ordinary semantic path, then replayed the
structural validator independently over every checked function. For each
function it required exactly one root for every accepted subscript, exactly
one root for every discharged ordinary-call goal, and all eight ordered atomic
roots behind the five S11 relations of every counted statement. Failed and
non-discharged outcomes retained no positive root. Every retained node was
reachable from one of those mandatory roots; parent IDs preceded children and
proved the exact typed conclusion; every retained proof-producing event was
referenced by a retained node; and recomputed root classes, nodes, real parent
edges, events, paths, components, capacity slots, depths, and bytes matched the
stored metrics.

| Frozen unit | functions | bounds roots | call roots | counted statements / semantic S11 / atomic roots | total `R` | conclusion shapes (bound / opaque / projected / contradiction) |
|---|---:|---:|---:|---:|---:|---:|
| UTF-8 | 2 | 33 | 1 | 0 / 0 / 0 | 34 | 33 / 0 / 1 / 0 |
| SHA-256 | 9 | 9 | 0 | 3 / 15 / 24 | 33 | 33 / 0 / 0 / 0 |
| four-source DEFLATE | 17 | 33 | 3 | 0 / 0 / 0 | 36 | 33 / 0 / 3 / 0 |
| wfgrep | 6 | 11 | 0 | 0 / 0 / 0 | 11 | 11 / 0 / 0 / 0 |
| **Aggregate** | **34** | **86** | **4** | **3 / 15 / 24** | **114** | **110 / 0 / 4 / 0** |

Here `S` is retained unique proof nodes, `P` is real retained parent edges,
`E` is retained proof-producing events, and `R` is mandatory roots. “Path
events” counts exact retained event occurrences carrying a `NodePath`.
`sum(len)` is their logical component utilization. `C` sums their independently
capacity-charged `u32` component slots; repeated path identity is charged again
at each exact event occurrence. Path depth and proof depth are separate.

| Frozen unit | `S` nodes | `P` edges | `E` events | path events | `sum(len)` | `C` slots | max path | max proof depth | ledger-owned bytes |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| UTF-8 | 76 | 82 | 7 | 6 | 30 | 30 | 6 | 3 | 6104 |
| SHA-256 | 78 | 67 | 13 | 10 | 48 | 48 | 6 | 4 | 6680 |
| four-source DEFLATE | 147 | 124 | 79 | 53 | 418 | 418 | 14 | 6 | 15340 |
| wfgrep | 73 | 82 | 38 | 16 | 210 | 210 | 31 | 16 | 7668 |
| **Aggregate** | **374** | **355** | **137** | **85** | **706** | **706** | **31** | **16** | **35792** |

Event pruning left no orphan event. Because each retained node names at most
one event and every retained event is named by at least one retained node, the
measured `E <= S` invariant holds per unit and in aggregate (`137 <= 374`).
Each event owns at most one path payload, so the event buffer is charged to
`S`, while variable path storage is charged separately to `C`. In these frozen
units logical utilization happened to equal capacity (`sum(len) = C = 706`);
that observed equality is not assumed by the bound.

The byte reconstruction independently summed outer node, event, root, and
depth buffer capacities at their Rust element sizes, every retained join's
predecessor-buffer capacity, and every retained event path's component-buffer
capacity. The last term was exactly `C * size_of::<u32>() = 706 * 4 = 2824`
bytes. The reconstruction reproduced 6104, 6680, 15340, and 7668 bytes and the
35792-byte aggregate exactly. “Ledger-owned bytes” excludes
`DerivationInventory`, nested term and goal allocations, outcome containers,
the cleared interner, and transient `FactState`/`ClosedState` scratch; it is
not the complete checked representation or process footprint.

The structural bound is therefore `O(S + P + R + C)`. One dense arena interns
proof nodes while existing source, closure, join, and materialization work
runs. Each node carries only fixed parents or the real predecessor vector at a
join; the root vector has one record per mandatory occurrence, with eight fixed
records per counted statement. Finalization performs one traversal from roots,
one dense remap, and event pruning. There is no DAG copy per root, extra
ledger-owned copy of one event's path, retained full program-point state,
query-triggered reclosure, second semantic walk, certificate, serialization,
portable identity, cache/replay, ProofFlow, shadow verifier, or new lowering
path. The counted preheader records the one already-required closed snapshot
before continuing kills; body roots name the same-walk S11 source parents.

The corrected temporary audit passed 1/1 in 325.20 seconds and was deleted.
The restored test source hashed
`94a6fd9579d163fd5ee9b72aa41f5d6550db26c664fb05041c7768aa74a05e0c`,
and the candidate worktree and index were clean. The focused entailment suite
then passed 112/112 in 364.61 seconds. It covers exact source identity, kills,
joins, mixed and all-contradictory states, ordinary and counted loops,
substitution, concrete generics, forward and mutual recursion, and frozen-unit
root completeness. Root deletion, duplication, path/relation corruption,
invalid and killed-parent substitution, and snapshot-kind mutation each failed
at the intended audit boundary; two twenty-run determinism controls remained
byte-identical.

### Exclusive release compile time and peak memory

The platform was macOS 26.5.2 build 25F84, Darwin 25.5.0, arm64; rustc 1.97.1
(`8bab26f4f68e0e26f0bb7960be334d5b520ea452`, LLVM 22.1.6), cargo 1.97.1
(`c980f4866141969fab6254a680546a277789d6f0`), and Apple clang 21.0.0.
The already-built release binaries were fixed before and after measurement:
baseline SHA-256
`44268827e7e715dc269aba3cf99bc4ae0b75eda9e470e164d40eb5e6f1435a9d`
at 2,786,720 bytes and candidate SHA-256
`a08959b8aa410724720dadbc2be043b7eecf22323a4e78f70be1e83c5ee43d9b`
at 2,874,832 bytes.

The sole valid session is
`/Users/bytedance/do_not_scan/whitefoot-0056-cost-rerun-20260813T230406Z`.
Two interval-separated pre-session quiet observations were made outside the
formal session and are contextual, unsealed checks. The formal session then
recorded one hash-sealed session-start checkpoint plus start, between, and end
checkpoints for each of 32 baseline/candidate pairs: 97 checkpoints total.
Those observations detected no Whitefoot compiler, Cargo, rustc, non-`clangd`
clang, or make competition. They are sampled evidence, not an assertion that
continuous absence was observable between checkpoints; any competition the
orchestrator detected at a checkpoint would have invalidated the whole
session.

Every measured invocation used this explicit sterile environment, then the
real executable-producing path from the relevant side's exact repository root:

```text
/usr/bin/env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin \
  TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0056-cost-rerun-20260813T230406Z/tmp \
  LANG=C LC_ALL=C \
  /usr/bin/time -l -o TIME ABSOLUTE_WHITEFOOTC -o UNIQUE_PROGRAM SOURCES...
```

The fixed unit order was UTF-8, SHA-256, the exact four-source DEFLATE vector,
then wfgrep. Each unit had a baseline warmup, candidate warmup, and seven
strict baseline-then-candidate formal pairs: 64 total invocations, of which 56
were formal samples. Every invocation has a unique directory containing its
exact cwd, revision, binary identity, environment, argv, ordered source paths
and hashes, UTC and nanosecond timestamps, stdout, stderr, raw `/usr/bin/time`
output, executable, exit status, sizes, hashes, and parsed real/user/sys/RSS.
All 64 exited zero. UTF-8, SHA-256, and wfgrep stdout and stderr were empty.
All 16 DEFLATE stderr files were byte-identical and contained exactly the same
five existing CLM-2 redundancy advisories, in order, naming
`count_slot_in_counts`, `validate_slot_in_counts`,
`offsets_slot_in_offsets`, `offsets_slot_in_counts`, and
`ordered_symbol_in_lengths`. Output executable hashes were stable for each
unit and side.

Method 2 below reports independently recomputed formal-sample medians with
min–max in parentheses, followed by candidate-minus-baseline absolute and
percentage deltas. Wall, user, and system times are seconds; maximum RSS is
Darwin bytes. Percentage deltas use the displayed metric's baseline median.

| Unit | metric | baseline median (min–max) | candidate median (min–max) | absolute delta | percentage delta |
|---|---|---:|---:|---:|---:|
| UTF-8 | wall | 2.86 (2.82–2.92) | 4.67 (4.61–4.70) | +1.81 | +63.29% |
| UTF-8 | user | 2.83 (2.77–2.87) | 4.65 (4.59–4.69) | +1.82 | +64.31% |
| UTF-8 | sys | 0.03 (0.02–0.03) | 0.03 (0.03–0.03) | +0.00 | +0.00% |
| UTF-8 | RSS | 40681472 (40648704–40796160) | 43515904 (41271296–44466176) | +2834432 | +6.97% |
| SHA-256 | wall | 0.52 (0.50–0.53) | 0.70 (0.68–0.71) | +0.18 | +34.62% |
| SHA-256 | user | 0.51 (0.50–0.52) | 0.69 (0.67–0.69) | +0.18 | +35.29% |
| SHA-256 | sys | 0.02 (0.02–0.02) | 0.02 (0.02–0.03) | +0.00 | +0.00% |
| SHA-256 | RSS | 45400064 (45268992–45449216) | 45350912 (45334528–45465600) | -49152 | -0.11% |
| DEFLATE | wall | 1.95 (1.91–2.00) | 3.13 (3.09–3.20) | +1.18 | +60.51% |
| DEFLATE | user | 1.94 (1.89–1.98) | 3.11 (3.07–3.16) | +1.17 | +60.31% |
| DEFLATE | sys | 0.02 (0.02–0.03) | 0.03 (0.03–0.03) | +0.01 | +50.00% |
| DEFLATE | RSS | 55459840 (55214080–55607296) | 55361536 (55066624–55607296) | -98304 | -0.18% |
| wfgrep | wall | 2.74 (2.70–2.82) | 3.93 (3.89–4.06) | +1.19 | +43.43% |
| wfgrep | user | 2.72 (2.69–2.81) | 3.89 (3.86–4.02) | +1.17 | +43.01% |
| wfgrep | sys | 0.02 (0.02–0.03) | 0.04 (0.04–0.05) | +0.02 | +100.00% |
| wfgrep | RSS | 50216960 (49889280–50249728) | 71041024 (70811648–77496320) | +20824064 | +41.47% |

Wall time remains platform-dependent, but the exclusive paired run has narrow
spread and user-time deltas agree in direction and magnitude. No numeric
slowdown or RSS budget was owner-approved, so these measurements are reported
rather than judged against an invented threshold.

The 64 sealed session run records are indexed by `runs.jsonl` SHA-256
`c9281493c4905551be46c08ecf08192b83cd9b91a34ccb3e500681b287ce04c9`;
the independently recomputed `summary.json` hashes
`b2bc83350d81a983699033c3fd4b160fde2856935f960617b32577ceaad510f4`;
the terminal `session.json` hashes
`343af8e443c2887efe1c42052eaec8f755d8c321438c580faddb08772cdd0e37`.
The sorted 484-entry artifact manifest hashes
`2e45b6f1c8908eaed4c0d337aef33a1b5db8015a5c384653fd37348744dc5382`,
and its one-line self-hash file hashes
`1cf8fb4cc7297020009acb78c9c32481b881038bef3ea5b097bbb7c5eebce996`.
Both `shasum -a 256 -c` checks and an independent 64-run structural audit
passed.

The mock-validated scratch orchestrator that created the manifests and enforced
the session assertions hashes
`e3bf4c0b4b869414b0d727fead1c20b38abd0704cc3d295a929dca1763674c71`.

Two earlier formal session identities are explicitly withdrawn and are not
measurements: one stopped before its first snapshot and one after only its
session-start snapshot, both with zero compiler runs, due solely to scratch
Ruby 2.6 incompatibilities. Their withdrawn identities are
`whitefoot-0056-cost-rerun-20260813T225458Z` and
`whitefoot-0056-cost-rerun-20260813T225944Z`. A separate disposable MOCK
session `whitefoot-0056-cost-rerun-mock-20260813T230310Z` exercised all
64 directory, manifest, result, parsing, ordering, competition-detection,
hash, and terminal-index paths without invoking either real compiler. Neither
the zero-run attempts nor MOCK values contribute to the Method 2 table. The
older concurrently collected audit at SHA-256
`ae254ef35baf1a50a71b6b3b172636df19dd222a59f78876a38a3aaa29cbbe07`
and section at SHA-256
`6b3120e43e51096742d43ecdce114d046779198e50c4552e9b05ec7185454704`
are also withdrawn; only the exclusive formal session supplies cost figures
for this evidence.

### Acceptance, behavior, and gates

The exclusive release comparison preserved acceptance and diagnostic bytes.
The baseline and candidate real-program suites each preserved all 30 source,
semantic, LLVM, execution, output/error, cleanup, effects, facts-on/off, and
required-check oracles. Final candidate validation was:

- focused entailment: 112 passed, 0 failed, 606 filtered, 364.61 seconds;
- compiler gate: green, including 718/718 core tests in 730.16 seconds and
  30/30 real-program tests in 1633.08 seconds;
- repository gate: green, including 23/23 compiler-independent runner tests,
  131/131 covered specification rules with none uncovered, 718/718 core tests
  in 596.08 seconds, and 30/30 real-program tests in 1303.36 seconds;
- independent native adapter: the existing read-only adapter completed in
  266.09 seconds with `Pass=409  Fail=1  Skip=13`; its only failure remained
  the recorded `own3-pos-outlives-store` A3 case reaching
  `Unsupported(RegionsAndBorrows)` instead of `Run(0)`.

The baseline real-program oracle independently passed 30/30 in 791.70
seconds. After the exclusive measurement and corrected temporary audit, the
task, candidate, and baseline worktrees were clean at their exact revisions;
every frozen identity and both binary identities above were reverified.

The active specification, frozen sources, compiler production bytes,
protected conformance evidence, and gate wiring were not changed by measurement
or audit.
The retained DAG remains private checked-program state and is not a loadable or
independently authoritative artifact.

## Stage 8a complete caller synthesis (2026-08-13)

This ordinary research result closes the Stage 8a caller audit at claimed
revision `ccecd40d2f9fb33f97c5aed1626875ef3e989375`. Its exact premise is the
closure commit `30b0ccc10d394dcce3403aaf49d149aea82f741d`, which installs the
terminal-positive 0051 and 0052 local-witness sections and independently
records task 0056 terminal-positive. No earlier caller table was reused.

The result is **PASS**. The fresh checked-tree census is exactly fourteen
`read_bits` calls and twenty `append_slice` calls. Every hypothetical
formal/result substitution is well typed. All fourteen `read_bits` relations
reach the `Ok` payload and expire at the current mutable-delivery seam. The
append admission map is exactly 19 discharged and 1 unproved, with no
refutation; the sole unproved row is wfgrep's separator after the successful
host-copy scalar update. No fourth flow class or additional repair appears.

This is a structural synthesis of future facts, not evidence that v0.27
already has postconditions. In particular, the fourteen `read_bits` entries
below are relation-survival and delivery gaps, not existing FN-8 call goals.
Nothing in this section changes language acceptance, lowering authority, a
required runtime check, or a protected compliance artifact.

### Frozen inputs and deterministic checked census

The active specification and all five real sources reproduced the Current
Plan identities:

| Input | SHA-256 |
| --- | --- |
| `spec/kernel-spec.md` | `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f` |
| `tests/programs/raw_deflate.wf` | `5e87885a519de539736b0ad6a619a2c92bdf659623e3f39ded31116e63adb585` |
| `tests/programs/raw_deflate_dynamic.wf` | `2606ae0b81039b0ecc787df2fa9cb87279ba44663744540d48f6abeea984d4c5` |
| `tests/programs/raw_deflate_dynamic_decode.wf` | `72129284c60a6eacbe2bb86d7d3a82375ed2270ea457c49d8e1db95064fc960f` |
| `tests/programs/raw_deflate_boundary.wf` | `3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4` |
| `tests/programs/wfgrep.wf` | `a1e49bcb9ffd353e707d4bbafe1eb4a2b634b9177c6916b2ff7b503ec5dff0bd` |

The direct source enumerations were:

```text
rg -n 'match read_bits<' tests/programs/raw_deflate.wf tests/programs/raw_deflate_dynamic.wf tests/programs/raw_deflate_dynamic_decode.wf tests/programs/raw_deflate_boundary.wf
rg -n '= append_slice<' tests/programs/raw_deflate_boundary.wf tests/programs/wfgrep.wf
```

They returned 14 and 20 source occurrences respectively. A disposable
in-crate checked-tree walk then visited every concrete function body in the
four-source raw-DEFLATE bundle and the standalone wfgrep unit. It asserted:

- `read_bits` count actuals are `u32`; the `Ok` payload, payload read, and
  outer delivery target are the same typed `u64` path described by each row;
- `append_slice` takes `buffer<u8>`, `u64`, and `slice<u8>` actuals, returns
  `u64`, and writes that result directly to the same `u64` binding supplied as
  `filled`; and
- the per-unit counts are raw-DEFLATE `14/8` and wfgrep `0/12`.

The exact focused command was run three times:

```text
env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-0053-cargo-tmp cargo test --manifest-path compiler/Cargo.toml --locked --offline stage8a_dump_real_caller_census -- --nocapture
```

Each invocation exited 0 and passed its one selected test. The three selected
test times were 151.89, 150.64, and 153.40 seconds. Normalizing each log with
`rg '^STAGE8A'` produced exactly 34 lines and the same SHA-256
`bbf7f3d05baa9910b70457ead1919355814a51ea06b472a90347830d3a2bd98c`;
pairwise `cmp` succeeded. The third run included the complete type and direct
receiver assertions above. Node paths in the tables are runtime checked-tree
coordinates and are meaningful only with the named unit and frozen source
identities; they are not portable identities.

In the tables, `bN` means checked `BindingId(N)`. Every listed caller has one
concrete checked instance whose symbol is the source function name; the region
arguments do not create duplicate type- or const-specialized instances.

The two real `append_slice` declarations remain distinct even though their
23-line bodies are byte-identical:

| Declaration | Lines | Body SHA-256 |
| --- | ---: | --- |
| `raw_deflate_boundary.wf:17-39` | 23 | `f6ce5cb725489c73ff74f4df78be1f9f95acb909eb673dfe717647326675941f` |
| `wfgrep.wf:121-143` | 23 | `f6ce5cb725489c73ff74f4df78be1f9f95acb909eb673dfe717647326675941f` |

Stage 8b must therefore verify and install the selected normal-result contract
on both declaration identities. Their identical flow is not a fourth caller
class.

### Typed `read_bits` delivery map: 14 relation-survival gaps

For this map, the hypothetical normal-result fact is
`Ok(result) => result < ishl.wrap(1_u64, count)`. After formal/result
substitution it relates a `u64` payload to a `u64` high value computed from a
`u32` actual. A literal count adds no mutable support. The two table-derived
count actuals add their named `u32` binding to the payload support.

Every current `Ok` arm performs `set outer = payload`. ENT-3 creates no RHS
equality, so the relation remains supported by the payload rather than the
outer target. The payload then leaves scope and the relation expires. The
following table names the first downstream source use only to fix the repair
boundary; none of those uses is being reclassified as an existing FN-8 goal.

| ID | Source call; concrete caller; checked call path | Actual count; payload -> target | Hypothetical relation and live support | First downstream use | Current disposition; one planned repair |
| --- | --- | --- | --- | --- | --- |
| R01 | `raw_deflate.wf:63`; `decode_fixed_symbol`; `[7,0,9,0,1,0,0,0,0,0]` | `1_u32`; `read_bit` b8 -> `bit` b7 | `read_bit < 2`; `{read_bit}` | `ior(code, bit)` at line 72 | delivery gap; immutable value-producing match |
| R02 | `raw_deflate.wf:178`; `decode_fixed`; `[9,0,4,0,3,0,3,0,5,0,0,0,0,0]` | `5_u32`; `distance_code` b17 -> `distance_symbol` b16 | `distance_code < 32`; `{distance_code}` | `copy_distance(... distance_symbol)` at line 188 | delivery gap; immutable value-producing match |
| R03 | `raw_deflate.wf:209`; `inflate`; `[10,0,7,0,1,0,0,0,0,0]` | `1_u32`; `final_bit` b6 -> `final_value` b5 | `final_bit < 2`; `{final_bit}` | `ieq(final_value, 1_u64)` at line 218 | delivery gap; immutable value-producing match |
| R04 | `raw_deflate.wf:221`; `inflate`; `[10,0,7,0,4,0,0,0,0,0]` | `2_u32`; `block_type_bits` b10 -> `block_type` b9 | `block_type_bits < 4`; `{block_type_bits}` | `ieq(block_type, 0_u64)` at line 230 | delivery gap; immutable value-producing match |
| R05 | `raw_deflate_dynamic.wf:31`; `decode_length`; `[13,0,16,0,0,0,0,0]` | `length_extra_count` b13; `extra` b15 -> `length_extra` b14 | `extra < ishl.wrap(1_u64, length_extra_count)`; `{extra, length_extra_count}` | `length_base +wrap length_extra` at line 40 | delivery gap; immutable value-producing match |
| R06 | `raw_deflate_dynamic.wf:56`; `copy_distance`; `[14,0,11,0,0,0,0,0]` | `distance_extra_count` b10; `extra` b12 -> `distance_extra` b11 | `extra < ishl.wrap(1_u64, distance_extra_count)`; `{extra, distance_extra_count}` | `distance_base +wrap distance_extra` at line 65 | delivery gap; immutable value-producing match |
| R07 | `raw_deflate_dynamic.wf:251`; `decode_table_symbol`; `[16,0,11,0,1,0,0,0,0,0]` | `1_u32`; `read_bit` b11 -> `bit` b10 | `read_bit < 2`; `{read_bit}` | `ior(code, bit)` at line 260 | delivery gap; immutable value-producing match |
| R08 | `raw_deflate_dynamic_decode.wf:26`; `decode_dynamic`; `[18,0,5,0,0,0,0,0]` | `5_u32`; `read_count` b4 -> `literal_count_bits` b3 | `read_count < 32`; `{read_count}` | `literal_count_bits +wrap 257_u64` at line 35 | delivery gap; immutable value-producing match |
| R09 | `raw_deflate_dynamic_decode.wf:43`; `decode_dynamic`; `[18,0,10,0,0,0,0,0]` | `5_u32`; `read_count` b10 -> `distance_count_bits` b9 | `read_count < 32`; `{read_count}` | `distance_count_bits +wrap 1_u64` at line 52 | delivery gap; immutable value-producing match |
| R10 | `raw_deflate_dynamic_decode.wf:55`; `decode_dynamic`; `[18,0,13,0,0,0,0,0]` | `4_u32`; `read_count` b14 -> `code_count_bits` b13 | `read_count < 16`; `{read_count}` | `code_count_bits +wrap 4_u64` at line 64 | delivery gap; immutable value-producing match |
| R11 | `raw_deflate_dynamic_decode.wf:78`; `decode_dynamic`; `[18,0,17,0,1,0,7,0,0,0,0,0]` | `3_u32`; `read_length` b24 -> `length_bits` b23 | `read_length < 8`; `{read_length}` | `cvt<u64, u8>(length_bits)` at line 87 | delivery gap; immutable value-producing match |
| R12 | `raw_deflate_dynamic_decode.wf:151`; `decode_dynamic`; `[18,0,26,0,1,0,5,0,3,0,4,0,0,0,0,0]` | `2_u32`; `read_repeat` b52 -> `repeat_bits` b51 | `read_repeat < 4`; `{read_repeat}` | `repeat_bits +wrap 3_u64` at line 160 | delivery gap; immutable value-producing match |
| R13 | `raw_deflate_dynamic_decode.wf:186`; `decode_dynamic`; `[18,0,26,0,1,0,5,0,3,0,14,0,1,0,0,0,0,0]` | `3_u32`; `read_repeat` b64 -> `repeat_bits` b62 | `read_repeat < 8`; `{read_repeat}` | branch join, then `repeat_bits +wrap repeat_base` at line 215 | delivery gap; value-producing match plus value-producing `if` |
| R14 | `raw_deflate_dynamic_decode.wf:204`; `decode_dynamic`; `[18,0,26,0,1,0,5,0,3,0,14,0,5,0,0,0,0,0]` | `7_u32`; `read_repeat` b68 -> `repeat_bits` b62 | `read_repeat < 128`; `{read_repeat}` | branch join, then `repeat_bits +wrap repeat_base` at line 215 | delivery gap; value-producing match plus value-producing `if` |

R13 and R14 are two calls but one outer branch join. The short arm's
`read_repeat < 8` entails the common weaker `read_repeat < 128`; the long arm
already has that relation. The smallest repair therefore uses the same
immutable-delivery rule first for each value-producing match and then for a
value-producing `if` that gives one `u64` on both reaching edges with the
common `repeat_bits < 128` relation. It does not require general assignment
equality or a new arithmetic source.

For R01-R12, replacing the mutable delivery with an immutable value-producing
match moves the established relation from the payload copy atom to the direct
receiving binding while ordinary scope kills remove every other arm-local
support. For R13-R14, the additional value-producing `if` is the already
planned shared form of the same bounded rule. These fourteen source repairs,
and no others, are the complete read-side caller boundary.

### Typed `append_slice` admission map: 19 discharged, 1 unproved

The hypothetical admitted-domain requirement at each call is
`filled <= len(deref(destination))`. The hypothetical normal-result relation
is `result <= len(deref(destination))`, instantiated directly onto the same
`u64` receiving binding that supplied `filled`. This table models that one
future direct-result rule only; it is not a general fact transfer for `set`.
The established relation is supported by the receiving `length` binding and
the destination buffer's length projection. The moved message slice is not
support and may leave scope without killing the relation. Destination element
writes preserve the length projection; a scalar write to `length` kills it.

Rows are ordered first by compilation unit, then source occurrence. `D` means
the future requirement at that call is discharged. `U` means unproved. The
six reason rows after A10 are a staged synthesis result: they are discharged
only after A10's one named repair has re-established the invariant and the
separator's direct result has published it.

| ID | Declaration; source call; caller; checked call path | Actual regions and checked receiver | Exact pre-support and intervening flow | Result |
| --- | --- | --- | --- | --- |
| A01 | raw-boundary declaration; line 122; `publish_reason`; `[31,0,6,0,1,0,1,0,0,0,1,0]` | `'usage_append/'usage_view`; report b1; length b3; view b4 | initial `0 <= len(report)`; result or false path preserves the invariant at the join | D |
| A02 | raw-boundary declaration; line 130; `publish_reason`; `[31,0,7,0,1,0,1,0,0,0,1,0]` | `'unreadable_append/'unreadable_view`; report b1; length b3; view b5 | prior joined `length <= len(report)`; view scope exit is irrelevant | D |
| A03 | raw-boundary declaration; line 138; `publish_reason`; `[31,0,8,0,1,0,1,0,0,0,1,0]` | `'empty_append/'empty_view`; report b1; length b3; view b6 | prior joined `length <= len(report)` | D |
| A04 | raw-boundary declaration; line 146; `publish_reason`; `[31,0,9,0,1,0,1,0,0,0,1,0]` | `'too_large_append/'too_large_view`; report b1; length b3; view b7 | prior joined `length <= len(report)` | D |
| A05 | raw-boundary declaration; line 154; `publish_reason`; `[31,0,10,0,1,0,1,0,0,0,1,0]` | `'truncated_append/'truncated_view`; report b1; length b3; view b8 | prior joined `length <= len(report)` | D |
| A06 | raw-boundary declaration; line 162; `publish_reason`; `[31,0,11,0,1,0,1,0,0,0,1,0]` | `'malformed_append/'malformed_view`; report b1; length b3; view b9 | prior joined `length <= len(report)` | D |
| A07 | raw-boundary declaration; line 170; `publish_reason`; `[31,0,12,0,1,0,1,0,0,0,1,0]` | `'output_full_append/'output_full_view`; report b1; length b3; view b10 | prior joined `length <= len(report)` | D |
| A08 | raw-boundary declaration; line 178; `publish_reason`; `[31,0,13,0,1,0,1,0,0,0,1,0]` | `'write_failed_append/'write_failed_view`; report b1; length b3; view b11 | prior joined `length <= len(report)`; final result reaches `publish_all` | D |
| A09 | wfgrep declaration; line 198; `report_failure`; `[16,0,6,0,1,0,0,0,1,0]` | `'prefix_append/'report_prefix`; report b2; length b5; view b6 | initial `0 <= len(report)`; direct result establishes `length <= len(report)` | D |
| A10 | wfgrep declaration; line 219; `report_failure`; `[16,0,8,0,1,0,0,0,1,0]` | `'separator_append/'report_separator`; report b2; length b5; view b11 | A09 relation survives the host-copy element write; `set length = length +wrap copied` kills it; S7 has no variable-plus-variable replacement; the error arm preserves it but the match join cannot | **U** |
| A11 | wfgrep declaration; line 228; `report_failure`; `[16,0,10,0,2,0,1,0,0,0,1,0]` | `'reason_missing_append/'reason_missing_view`; report b2; length b5; view b13 | staged after A10 repair and separator result; true result or false path preserves the same relation | D, conditional on A10 repair |
| A12 | wfgrep declaration; line 237; `report_failure`; `[16,0,11,0,2,0,1,0,0,0,1,0]` | `'reason_denied_append/'reason_denied_view`; report b2; length b5; view b14 | staged prior joined relation | D, conditional on A10 repair |
| A13 | wfgrep declaration; line 246; `report_failure`; `[16,0,12,0,2,0,1,0,0,0,1,0]` | `'reason_directory_append/'reason_directory_view`; report b2; length b5; view b15 | staged prior joined relation | D, conditional on A10 repair |
| A14 | wfgrep declaration; line 255; `report_failure`; `[16,0,13,0,2,0,1,0,0,0,1,0]` | `'reason_path_append/'reason_path_view`; report b2; length b5; view b16 | staged prior joined relation | D, conditional on A10 repair |
| A15 | wfgrep declaration; line 264; `report_failure`; `[16,0,14,0,2,0,1,0,0,0,1,0]` | `'reason_long_append/'reason_long_view`; report b2; length b5; view b17 | staged prior joined relation | D, conditional on A10 repair |
| A16 | wfgrep declaration; line 272; `report_failure`; `[16,0,15,0,1,0,1,0,0,0,1,0]` | `'reason_other_append/'reason_other_view`; report b2; length b5; view b18 | staged prior joined relation; final result reaches `publish_all` | D, conditional on A10 repair |
| A17 | wfgrep declaration; line 329; `main`; `[17,0,15,0,2,0,1,0,1,0,1,0]` | `'startup_usage/'startup_usage`; report b7; length b16; view b17 | branch-local initial `0 <= len(report)`; direct result establishes the relation | D |
| A18 | wfgrep declaration; line 334; `main`; `[17,0,15,0,2,0,2,0,1,0,1,0]` | `'startup_pattern/'startup_pattern`; report b7; length b16; view b18 | alternate branch-local initial relation; both branches join with the same result relation | D |
| A19 | wfgrep declaration; line 568; `main`; `[17,0,26,0,3,0,1,0,1,0,1,0]` | `'fail_pipe_view/'fail_pipe_view`; report b7; length b83; view b84 | branch-local initial `0 <= len(report)`; direct result establishes the relation | D |
| A20 | wfgrep declaration; line 573; `main`; `[17,0,26,0,3,0,2,0,1,0,1,0]` | `'fail_write_view/'fail_write_view`; report b7; length b83; view b85 | alternate branch-local initial relation; both branches join with the same result relation | D |

The only append-side caller repair is A10. On the successful host-copy path,
Stage 8b must bind the candidate next length, take an explicit
value-producing branch on `next_length <= len(deref(report))`, give the
candidate only on the true edge, and otherwise give the prior length exactly
as the existing failed-copy path leaves it. The result is one immutable
length for the separator. This uses a real value branch and the same bounded
delivery rule as the read side. It does not add variable-addition S7, a
trusted system arithmetic premise, a writer assertion, or general assignment
equality.

### PASS synthesis and exact Stage 8b boundary

The three admitted flow classes are exhaustive:

1. fourteen `Ok payload -> set outer -> payload scope exit` paths, repaired
   by the bounded immutable-delivery rule shared by value-producing match and
   value-producing `if`;
2. twenty direct `append_slice` normal results instantiated onto their direct
   receiving targets, with ordinary scope and branch joins; and
3. A09's append relation through host-copy element writes, then killed by the
   variable-addition scalar update before A10.

The smallest successful Stage 8b caller boundary is consequently fixed:

- install only the two measured unsigned bit sources needed to verify the
  `read_bits` normal-result relation;
- use the measured counted `append_slice` body and admitted-domain
  requirement, and verify the selected normal-result relation on both
  distinct real declarations;
- add verified normal-return postconditions and the one bounded immutable
  value-delivery rule needed by the fourteen read calls; and
- make only the one wfgrep host-copy value-branch repair described above.

No solver, third fact source, loop induction, arithmetic-expression fact
family, recognizer, general `set` equality, variable-offset or
variable-plus-variable S7 rule, trusted writer assertion, or additional
consumer repair is required. No row is refuted. The task's stop conditions
therefore did not fire.

The disposable checked-tree edit was removed after capture.
`compiler/src/semantic/tests/entailment.rs` returned to SHA-256
`94a6fd9579d163fd5ee9b72aa41f5d6550db26c664fb05041c7768aa74a05e0c`.
The restored raw-DEFLATE program selection passed 3/3 in 846.32 seconds, and
the restored wfgrep selection passed 9/9 in 207.48 seconds. The final
`make check` passed all 28 recorded specification identities, conformance
structure 23/23, specification coverage 131/131 with none uncovered, compiler
tests 718/718 in 468.08 seconds, and real programs 30/30 in 1083.66 seconds.
Rustdoc completed with warnings denied in 4.23 seconds. The run ended with
`WHITEFOOT COMPILER GATE GREEN` and
`WHITEFOOT GATE GREEN (active compiler + independent evidence)`.

All task-local probe logs, the dedicated Cargo temporary directory, and the
isolated worktree's generated compiler target tree were removed after the
successful gate. This PASS permits preparation of the separate Stage 8b
candidate; it does not approve or activate any future specification or
protected-conformance byte.

## Installed v0.28 Stage 8b acceptance (2026-08-15)

The owner-approved Stage 8b candidate is now installed as active v0.28 at
`spec/kernel-spec.md`, SHA-256
`08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`.
The outgoing v0.27 authority is preserved byte-identically at
`spec/kernel-spec-v0.27.md`, SHA-256
`bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`.

The installed five-source identities are, in compilation order:

- `raw_deflate.wf`
  `6da69f4dfaa3906c8516a4b0eb3f113b96b8c406084ac3c16ffb92514098bdf2`;
- `raw_deflate_dynamic.wf`
  `87099254c197460733a3a4661b0a5e0dd6290936e4647634ceed0134043b0b84`;
- `raw_deflate_dynamic_decode.wf`
  `8604a81550083bdaea85e5da0aaf798a75e14beb7626a2f21d08975ae88d071d`;
- `raw_deflate_boundary.wf`
  `c5044c5db980e1d2c14b5c3731a153351a58d61d84aec81880e355707c2a2f84`;
- `wfgrep.wf`
  `44db13e238b00260ec4f23a60be85db700d22902c6657a406ec13bc87b6a4889`.

The ordinary checked path retains exact complete, unasserted, and S4-blinded
proof roots for all fourteen `read_bits` selected-payload receiver routes.
The raw-DEFLATE boundary declaration retains eight direct `append_slice`
receiver routes per view, while the distinct wfgrep declaration retains twelve
per view. Wfgrep's A10 host-copy boundary alone uses the installed `value_if`
delivery join to select `candidate_length` or the prior length; the separator,
A11--A16, and final publication all consume that one `bounded_length` result.
No `value_match` delivery, general assignment equality, variable-addition S7,
runtime fallback, or second proof path was introduced.

The two clause-stripped append controls preserve the invalid-domain behavior:
at capacity 3 with `filled = 4`, both empty and nonempty text return 4, leave
all three destination bytes unchanged, and produce no output. The existing
raw-DEFLATE and wfgrep suites preserve success output, mapped errors, cleanup,
effects, process status, required runtime checks, and facts-off behavior. The
single frozen-real owning run validates the exact fourteen read routes, eight
raw append routes, twelve wfgrep append routes, and A10 ancestry.

The protected corpus is additive: manifest SHA-256
`8fada5059b57d563ab00a1c1c305dcd5810201ea2c507ee00a4137102bfc18f3`,
437 cases, 30 unchanged annotations, and 132/132 rule coverage. The native
adapter reports `Pass=423 Fail=1 Skip=13`; the sole failure remains the
pre-existing OWN-3 `own3-pos-outlives-store` unsupported boundary.

The installed activation checks pass archive integrity for 29 recorded
specifications, archive-to-active native grammar identity at 73 productions,
90 decisions, and 97 terminal predicates, conformance structure 23/23, and
coverage 132/132 with none uncovered. `make -C compiler check` is green with
808/808 library tests, grammar 9/9, generated grammar tables 1/1, migration
36/36, specification integrity 10/10, canonical corpus 3/3, and real programs
32/32; the conformance adapter integration remains deliberately ignored in
that gate and is run separately. Rustdoc passes with warnings denied. The
native specification tool reports v0.28 at the approved digest, 132 rules, and
20 unbroken activation entries.
