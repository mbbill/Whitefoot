> Promoted verbatim on 2026-08-23 from the round-3 debate corpus in the lead's
> scratch, because `../DESIGN.md` and `docs/ongoing/0078-loop-permission.md`
> cite it as the standing evidence for the deferred join-arbitration
> alternative, and a claim whose evidence has evaporated is an assertion.
> Nothing below is edited except three absolute machine-local paths in the
> Reproduction appendix, replaced by `<repository>` and `<the lead's
> scratch>` because a repository artifact carries no machine path. It is
> deleted when the arbitration question it defends is settled or superseded.

# d1-defense — DEFENSE of the Permission Framework against a1-semantics

2026-08-21. Task d1 of the round-3 structured debate on
`proposal-permission-framework.md`. I defend the framework's **best** version,
not its v0 wording. Where a1 is right I concede and pay with an amendment;
where a1 is right about less than it claims I say exactly what survives.

Evidence labels: **[verified]** = exact `spec/kernel-spec.md` line read by me;
**[measured]** = I compiled and/or ran it and read the output;
**[derived]** = follows from verified facts by the stated reasoning;
**[argued]** = judgment, contestable.

Repo untouched (read-only; `whitefootc` invoked from
`compiler/target/release/`). New probes live under `debate/probes/`.

## Verdict table

| # | a1's target | Disposition | Why |
|---|---|---|---|
| 1 | R-a is a degradation with no burden-bearer | **CONCEDE (fully)** | a1 is right on all three sub-points. R-a is withdrawn. |
| 2 | R-b/R-c miss B1; body-local termination unsound | **CONCEDE (a)(b)(c)** / **NARROW the consequence** | Both are withdrawn; §7's "B1 now" survives via Amendment A, which needs no termination judgment. |
| 3 | §4 "1 claim ⇒ free overlap" is an overclaim | **NARROW** (diagnosis conceded, sharpened; conclusion refuted) | a1 understates: §4's gate is not even a sound *proxy* for byte-identity — measured. Repair: demote §4 from legality to cost. |
| 4 | "immediate abort, no coordinator" is false | **CONCEDE the claim** / **NARROW the coordinator** | Phrase deleted. But no mutual exclusion and no cross-lane signalling is needed for correctness; the slot is one word. |
| 5 | Metatheorem load-bearing on DIAG-3 poverty; no composition | **NARROW** | Composition proof supplied (Lemmas A–D). Under Amendment A the trap-selection lemma no longer needs DIAG-3 poverty; the residual protected premise is narrower and already non-normative at spec:1968. |
| 6 | §10 aims at the wrong target; `allocates`, peak memory, external aliasing | **CONCEDE 6b, 6c, §10 retargeting** / **NARROW 6a** | a1's dichotomy for `allocates` is false: no §2 observable can depend on allocator state, so the TCB line (not P's W-set) is the correct repair — and a1's preferred repair costs 0 on the measured corpus anyway. |

Net: **two of a1's six objections retire proposal options outright (1, 2), one
forces a rewrite that makes the framework strictly stronger (3), two are
drafting/documentation defects with cheap exact repairs (4, 5), and one is
right about the gap but wrong about the fix (6).** None of them touches P's
memory core, which a1 concedes and which I do not need to defend.

---

## §0. Amendment A — the single change that answers 1, 2, 4, and most of 5

Four of a1's six objections are downstream of one v0 mistake: **§2 tries to buy
divergence-soundness with a termination judgment, and §4 tries to buy
trap-soundness with a claim-count gate.** Both are unnecessary. The correct
mechanism is already sitting in §5 and the proposal did not notice: *every
combination sits at a source-fixed join*, and a source-fixed join carries a
**total order on its lanes — the order the sequential elision executes them.**
Arbitrate the observable in that order and the strong law holds with **zero**
checker work.

Credit where due: g1-divergence already named this option **R-d** ("deferred
trap delivery at the join") with the Cilk/OpenCilk precedent, and priced its
bill as "EFF-4 and §4's no-coordinator claim"
[verified, `debate/g1-divergence.md:534,557`]. My contribution here is (i) the
exact arbitration rule including the divergence case, (ii) payment of the EFF-4
bill from spec text, (iii) the composition proof a1 says is missing, and (iv)
the demotion of §4 to a cost tier.

### Amendment A (replaces §2's R-a / R-b / R-c entirely)

> **[PAR-1] Elision-ordered join delivery.** Under actualization, an executed
> `claim` that fails in an overlapped lane produces its [DIAG-3] record into
> that lane's outcome slot and abandons the lane; abandoning a lane performs no
> unwinding and runs no release action [EFF-4]. Every source-fixed join carries
> the total order in which the sequential elision executes its lanes. The join
> resolves its outcome by scanning that order: at lane *i*, if lane *i* has not
> terminated the join does not resolve; if lane *i*'s outcome is a trap, the
> join's outcome is that trap; otherwise the scan proceeds to *i+1*. If every
> lane's outcome is normal, the join's outcome is the source-fixed combination
> of their results. A join's trap outcome is the outcome of the lane containing
> it. The program emits exactly one [DIAG-3] record — the outermost join's trap
> outcome — and aborts.
>
> An implementation may abandon lanes later in the order than a resolved trap;
> that is an optimization, not part of this rule.

The clause "if lane *i* has not terminated the join does not resolve" is the
whole divergence fix: an earlier diverging lane blocks the join forever, so the
overlapped run hangs exactly where the elision hangs. **No termination
predicate appears anywhere.**

Case check against a1's own probes [derived]:

| elision | lane 1 | lane 2 | rule resolves to | elision's outcome |
|---|---|---|---|---|
| `g1_siblings.wf` | diverges (`spin`) | traps (`trapper`) | never resolves ⇒ hang | hang ✓ |
| reversed | traps | diverges | lane 1's trap, immediately | trap ✓ |
| `d1_two_traps.wf` | traps `left_small` | traps `right_small` | lane 1's record | `left_small` ✓ [measured] |
| `a1_two_sites.wf` (no trap) | normal | normal | combination | same ✓ |

`d1_two_traps.bin` **measured**: elision prints exactly
`{"rule_id":"CLM-1","message":"left_small","function":"left","node_path":[0,0,3,0]}`
and exits 134. That is the observable Amendment A reproduces for every schedule
and that §4-as-written can only avoid by refusing to actualize.

### Amendment A pays the EFF-4 bill from spec text

a1-semantics-5 asserts in passing that "under R-d it holds but violates EFF-4."
**REFUTE, on EFF-4's normative sentences** [verified, spec:1428-1429]:

> "[EFF-4] Trap is abort: there is no unwinding and no post-violation language
> cleanup. The exact [DIAG-3] trap record is the sole mandatory post-violation
> language output."

Deferred delivery satisfies both literally. An abandoned lane is never unwound,
and it runs no release action — and that is not my reading, it is already
normative and independently stated: **"Release actions run only on normal
edges; a trap runs none and contributes nothing [EFF-4]"** [verified,
spec:1402]. Exactly one record is still emitted. EFF-4 constrains *what happens*
after a violation (no cleanup, one output); it states nothing about *when* the
record appears.

**Concession inside the refutation:** "Trap is abort" is a headline written for
the sequential case, and a strict reader can construe *abort* as immediate
process termination. So Amendment A must state the delivery point explicitly —
that is a spec change and therefore squarely inside the owner-approval boundary.
The honest position is: EFF-4's normative text does not forbid deferred
delivery, and the amendment removes the ambiguity. It is not "a direct conflict"
[argued against `g1-divergence.md:534`].

Trap latency, g1's other worry: under Amendment A the delay between a lane's
claim failing and the record appearing is bounded by the completion of exactly
the lanes that **precede it in the elision** — which is precisely the wait the
elision itself imposes on that statement, plus O(join depth) propagation. Not
"unbounded": *equal to the elision's* [derived]. The genuine cost is that a
trap winning in wall-clock time may be discarded — wasted work only.

### Amendment A supplies the composition metatheorem (a1-semantics-5's headline)

> **[PAR-2] Elision equivalence.** For every accepted program and every
> actualization schedule admitted by P, the law's `external`/`blocks` exclusion,
> and [PAR-1], the observable tuple — result values, trap-or-normal outcome,
> trap record bytes, external effect order — equals the sequential elision's.

*Proof, by induction on the source-fixed join tree.*

- **Lemma A (input identity).** P(1) no dataflow plus P(2) disjoint
  W×(W∪R) footprints under OWN-7 give: no lane reads storage a concurrent lane
  writes. Backed by OWN-7's conservative overlap relation, including
  "two subscripted places with the same resolved base overlap iff their offsets
  are not both literals with unequal values" and "formal-slice origins ... never
  establish that two actual sources are disjoint" [verified, spec:591-595];
  OWN-12's two-`&uniq`-arguments error [verified, spec:612]; OWN-13's sibling
  binder rule [verified, spec:618]; and EFF-2's boundary projection
  [verified, spec:1385-1407]. a1 concedes this half and I rely on the
  concession.
- **Lemma A′ (allocator neutrality).** See §6 below. The only state two lanes
  share outside P's footprint is the heap allocator and, when both rows carry
  `allocates(arena 'r)` at the same substituted `'r`, that region's bump
  allocator. **No observable in the law's tuple can depend on either.**
- **Lemma B (per-lane schedule invariance).** Given A and A′, a lane's outcome
  ∈ {normal(v), trap(r), divergence} is a function of its inputs alone, and its
  inputs equal the elision's at that statement. The only source of
  schedule-dependent input would be `external`/`blocks`, excluded by the law.
  **Divergence is handled here as an ordinary third outcome, not excluded** —
  this is exactly what R-a/R-b/R-c were trying and failing to do.
- **Lemma C (join arbitration = sequential composition).** [PAR-1]'s scan rule
  *is* the sequential semantics of `s1; s2; …; combine`: stop at the first
  diverging or trapping statement, otherwise combine. With Lemma B, the join's
  outcome equals the elision's outcome for its subtree.
- **Lemma D (external order).** No overlapped body carries `external`/`blocks`,
  so EFF-5's sequential external order [verified, spec:1431-1435] is untouched.
  **This is a scope limitation, not a discharged obligation** — a1-semantics-5(2)
  is right and I adopt its wording (see Amendment F).
- **Induction.** Joins nest and are source-fixed; each yields its subtree's
  elision outcome; the root yields the program's. ∎

That is the "no composition argument" gap closed, and it is closed *only*
because Amendment A makes divergence a lemma-B outcome rather than an open
clause. a1's claim that "there is no version of the framework in which the
composed metatheorem is both true and cost-free" is **REFUTED as stated**: this
version is true, and its cost is one spec clause plus a one-word-per-lane join
arbiter — not a termination analysis (R-b+), not a licensed degradation (R-a),
and not an EFF-4 violation (spec:1402/1428).

---

## a1-semantics-1 — R-a — **CONCEDE (fully)**

All three sub-points are correct and I add nothing to soften them.

- (a) A WF trap is a *defined* outcome — mandatory record bytes [verified,
  spec:1944-1967] and mandatory abort-with-no-cleanup [verified, spec:1428] —
  so hang→trap is not a refinement in CompCert's ordering. Correct.
- (b) R-a's guarantee is conditioned on an undecidable predicate that, by
  construction, **no party evaluates**: the checker does nothing (that is R-a's
  entire appeal) and the runtime is pre-absolved exactly on the inputs where the
  condition bites. Correct, and this is the sharpest form of the objection.
- (c) The asymmetry with spec:1424 — "Elimination of an unused pure call
  additionally requires a termination proof; v0 provides no termination checker,
  so unused pure calls are not eliminated" [verified] — is real and, in a spec a
  human must approve, disqualifying. Correct.

**Implication:** R-a is withdrawn from §2. **Cheapest repair:** Amendment A,
which achieves R-a's zero-checker-cost property *without* the weakening — the
strong law is preserved, not conditioned. The only surviving fragment of R-a's
motivation is its diagnosis (the divergence defect is real, x1-F2 confirmed by
`g1_siblings.wf`, which I recompiled independently: exit 0 [measured]).

## a1-semantics-2 — R-b/R-c — **CONCEDE (a)(b)(c); NARROW the consequence**

**(a) CONCEDE.** B1's termination measure is numeric. f1's catalog gives the
shape as `let mid = lo + (n / 2_u64)` with a `ilt(n, 4096_u64)` cutoff
[verified, `scenario-study/f1-permission-catalog.md:437-461`] — descent on
`hi − lo`, not into a matched `box` child. §2's parenthetical "it plausibly
covers … B1" is wrong, and R-b/R-c as worded would decline §7's one loop-side
target.

**(b) CONCEDE, and I reproduced it by execution rather than by reading.**
`a1_closure.wf`: `fold` recurses only into matched `box` children, is
`reads('r)`, has zero claims — textbook structural descent — and calls `weigh`,
whose `loop @l` breaks only on `acc == 7` or `acc < 7` while `acc` is never
updated. I compiled a leaf-weight-9 variant and ran it:

```
d1_closure_div.bin  -> exit 124 (timeout 5s)       [measured]
a1_closure.bin      -> exit 0  (leaves 3, 4)       [measured]
```

A body-local structural-descent judgment would whitelist `fold` and be wrong.
So the judgment must close over the call graph, and g1's measured closure
collapse (43/100 functions) is the price. Conceded.

**(c) CONCEDE.** R-c's body granularity is R-b's closure wearing a hat.

**NARROW — the consequence a1 draws does not follow.** a1's "what would change
my mind" asks for a statement that "the loop-side v0 is knowingly deferred until
R-b+ ships (making §7's 'B1 now' false today)." **Refused, with reason.** Under
Amendment A, B1 actualizes today: `scan`'s two recursive calls read one shared
buffer (`W = ∅`, overlapping reads licensed by OWN-5), P holds, and divergence
— should `scan` ever diverge — is preserved by the join, not judged by the
checker. §7's "B1 now" survives; what dies is the *termination-judgment branch*,
which g1 priced as the largest spec-token option of the four. **Withdrawing
R-b/R-c is a net reduction in spec surface, not a deferral of capability**
[derived].

## a1-semantics-3 — §4 eligibility — **NARROW (diagnosis conceded and sharpened; conclusion refuted)**

**Conceded and independently reproduced:** `a1_two_sites.wf` compiles
(exit 0 [measured]); P permits `left || right` (disjoint `own u64` footprints,
no dataflow, non-external); §4-as-written refuses it because the two claim sites
have distinct `(function, node_path)` pairs. The binding constraint is site
*diversity*, not count. §4's text is wrong and must be rewritten.

**a1 understates the defect.** §4's gate is not merely too strict — it is not a
sound *proxy* for the property it is trying to enforce (byte-identity of the
reported record). Measured, two ways:

1. **Different failing values, same site → identical bytes.**
   `d1_one_site.wf` (fails at 150) vs `d1_one_site_b.wf` (fails at 900):
   `cmp` → **BYTE_IDENTICAL** [measured]. Confirms a1's steelman.
2. **Two *distinct claim identities* → identical bytes.** A claim identity is
   `(concrete function instance, claim_stmt NodePath, claim name)`
   [verified, spec:2625], so `checked_add<u64>` and `checked_add<u32>` are
   **two** identities. Their records:

   ```
   {"rule_id":"CLM-1","message":"generic_sum_fits","function":"checked_add","node_path":[0,0,5,0]}
   {"rule_id":"CLM-1","message":"generic_sum_fits","function":"checked_add","node_path":[0,0,5,0]}
   cmp -> GENERIC_BYTE_IDENTICAL                                    [measured]
   ```

   because `function` is the *source* IDENT [verified, spec:1959] and
   `node_path` is the static production [verified, spec:1960]. §4 counts two
   identities and excludes; the bytes are identical and it need not.

So §4-as-written is **both too strict and unnecessary**. And the exclusion bites
hard: in the real-program corpus, **53 of 100 function declarations carry
`traps` in their row, across 241 claim sites in 24 programs** [measured,
`tests/programs/*.wf`]. Any parent folding two of those 53 is §4-ineligible as
written. a1's prediction ("outside self-recursive folds, near zero") is the
right order of magnitude.

**REFUTE the conclusion** that the *framework's* eligible set is tiny. Under
Amendment A the claim gate disappears from legality entirely, because the join
reports the elision's trap regardless of how many distinct sites are in play.

### Amendment B (replaces §4)

Claim structure becomes a **cost tier**, not an eligibility gate:

- **T0 — no reachable claim** in the overlapped bodies: no slot, plain
  fork/join, zero trap machinery.
- **T1 — all reachable claim identities emit identical record bytes**: the slot
  is **one bit**. This is where DIAG-3's information-poverty genuinely pays, and
  it pays *more* than §4 thought: identity-count is the wrong test (measurement
  2 above), byte-equality is the right one, and it is a compile-time decision.
- **T2 — otherwise**: the slot holds a **static site index (one word)**, because
  every DIAG-3 field is compile-time data — `rule_id` is the constant `CLM-1`
  [verified, spec:1958], `function` is a source IDENT [spec:1959], `node_path`
  is static [spec:1960], `message` is the claim's IDENT spelling and the
  justification STRING never appears [spec:1961]. The record table is a
  constant. **Legal, and still one word per lane.**

"Better checker ⇒ fewer claims ⇒ more code eligible" is deleted and replaced by
"better checker ⇒ more sites in T0/T1 ⇒ cheaper actualization." That gradient is
honest and still the AI writer's signal.

## a1-semantics-4 — "immediate abort, no coordinator" — **CONCEDE the claim; NARROW the coordinator**

**CONCEDE.** The phrase is false and is deleted. a1 is right that two lanes
reaching the failing site could each emit, and DIAG-3 fixes the output exactly —
"the displayed JSON object followed by exactly one byte `0x0A`" [verified,
spec:1952], "no extra whitespace or fields" [verified, spec:1955],
"Identical bound source bytes reaching the same failing claim site produce
byte-identical report bytes" [verified, spec:1967]. Two records or interleaved
bytes violate that. a1 is also right that the "win over R-d" §4 advertises does
not exist.

**NARROW — the coordinator a1 describes is not the one Amendment A needs, and
it is smaller.** a1 asks for "a single-writer latch on the mandatory record plus
a process-abort signal." Under [PAR-1] neither is required for correctness:

- **No mutual exclusion.** Each lane writes **only its own slot**; no two lanes
  write the same location. There is no race to exclude.
- **No cross-lane signalling for correctness.** The join reads slot *i* only
  after joining lane *i* — the ordinary fork/join happens-before edge. The
  early-abandon signal for lanes after a resolved trap is an optimization.
- **Exactly one emitter by construction.** No lane ever emits. Only the
  outermost join emits, once. The single-record obligation is discharged
  structurally, not by a latch.
- **The slot is one word** (Amendment B, T2).

**Also CONCEDE a1's closing point in full, and promote it.** a1 notes that safe
mid-write abandonment "rests on P (§3)." Correct, and it is a *premise of
[PAR-2]*, not an incidental dependency: an abandoned lane's partial writes are
unobservable only because P puts them in storage no surviving lane reads and the
process aborts before any further read. That premise is stated in Lemma A and
should be stated in the spec clause too. And spec:1402 makes the "no cleanup on
abandonment" half already-normative, so the amendment is aligned with EFF-4
rather than carving an exception from it.

**Amendment C (§4/§6 wording):** delete "Immediate abort, no coordinator."
Replace with: *"Lanes never emit. Each lane writes one outcome word; the
enclosing join reads it after the ordinary join edge; the outermost join is the
sole emitter. No mutual exclusion and no cross-lane signal is required for
correctness. Safe abandonment of a losing lane is a consequence of P and of
[EFF-4]'s no-release-on-trap rule (spec:1402); the join arbiter and lane
abandonment enter the accounted TCB."*

## a1-semantics-5 — the metatheorem — **NARROW**

**Composition:** answered in §0 (Lemmas A–D + induction). a1's charge that no
composition proof exists was true of the v0 document; it is now false of the
amended one, and a1's stronger claim (no true, cost-free version exists) is
refuted by construction.

**External-lane exclusion as scope, not solution:** **CONCEDE in full.** Lemma D
is easy because the payload was removed. a1's point (2) is adopted verbatim as
Amendment F below, together with a1-semantics-6's aliasing rationale (x1-F13):
the exclusion is load-bearing for *aliasing* as well as ordering, so the
external lane is unparallelizable under this law for soundness reasons, not
merely staging reasons.

**Trap-selection lemma and DIAG-3 poverty:** **NARROW, and the narrowing matters
a lot.** a1's structural claim is that the framework silently promotes DIAG-3's
byte-poverty from a diagnostic-quality decision to a parallelism-soundness
invariant, so that a future value/index/call-stack field would silently break
soundness. Against §4-as-written that is **correct** and it is the single best
finding in the attack: §4's whole trap argument is "the winner doesn't matter
because the bytes are the same," which is exactly a dependence on poverty.

Under Amendment A it is **no longer true**. The reported record is produced by
the *elision-designated* lane executing the *elision-designated* dynamic claim
occurrence (Lemmas B + C, by induction over nested joins). Therefore:

- A future DIAG-3 field carrying the **failing value** or a **subscript index**
  would still be byte-identical to the elision's, because it is the elision's
  occurrence that is reported. **Amendment A removes the coupling a1 found.**
- The **residual** protected premise is strictly narrower: DIAG-3 must not
  expose **worker/lane identity or the physical call stack**. And that is
  already non-normative today — "Dynamic call-stack attribution, artifact
  identity, successful-check reports … are not normative outputs"
  [verified, spec:1968].

**CONCEDE the documentation half regardless:** a coupling that is narrow is
still a coupling, and a human approver must be told. Amendment E below states
the protected premises explicitly, as a1 asks. Note the net direction: the
amended framework makes "better trap diagnostics" *safer* than the v0 framework
did, which is the opposite of the trajectory a1 projected.

## a1-semantics-6 — §10, `allocates`, peak memory — **CONCEDE 6b/6c/§10; NARROW 6a**

**§10 retargeting: CONCEDE and adopt.** a1 tried to build "OWN-7-disjoint yet
interfering" through memory and could not; I make no attempt to rescue the
headline falsifier. It is retired (Amendment G).

### 6a — `allocates` — NARROW

**Conceded:** §3's "`allocates(arena 'r)` handling: OPEN" is a real hole, and
§6's singular "**the** allocator" is wrong — box/buffer frees, arena bump
allocation, and the release path all run concurrently under overlap and none is
enumerated. a1 is right that the proposal has not chosen.

**Refuted: a1's dichotomy.** a1 says the choice is (i) `allocates(R)` enters P's
W-set at "a real expressiveness cost the proposal hides," or (ii) "a much larger
TCB." The second horn is correct and cheap; the first is unnecessary, because
**no observable in the law's tuple can depend on region-allocator state**
[derived from five verified facts]:

1. **Allocation cannot trap.** Unproved allocation-fit obligations are *source
   rejections*, not runtime checks: "unproved function, operation-domain,
   allocation-fit, bounds, and system-range obligations are source rejections"
   [verified, spec:1470], and "proved allocation operations … contribute no
   `traps`, because source acceptance precedes lowering and admits no runtime
   fallback" [verified, spec:1365]. So no `trap-or-normal` outcome is a function
   of allocator state. This is the load-bearing point and a1's probe does not
   address it — `a1_arena_twosib.wf`'s `fill_fits` claim is a *value* claim
   (`d = v +defined 1`), schedule-invariant under Lemma B, not a capacity claim.
2. **Exhaustion is not a language observable.** "Exhaustion during execution is
   inside the compiler/runtime/OS TCB boundary [SCOPE-3], not a language trap:
   it adds no source effect, produces no mandatory [DIAG-3] record"
   [verified, spec:725].
3. **No address is a value.** The operation table exposes no
   pointer-to-integer form; `reinterpret`'s domain is "equal-width primitive
   pairs: i8<->u8 … {i64,u64}<->f64" [verified, spec:777]. Allocation order is
   therefore invisible to result values.
4. **Overlap cannot change an arena's peak occupancy.** Arena storage "is
   released with its region [STOR-4]" [verified, spec:665] — there is no
   per-value free inside a region, so region occupancy is the *total* of the
   allocations, which overlap does not change, only reorders [derived].
5. **The rule cannot be evaded indirectly.** A function-typed value's row "is
   any subset of `allocates(heap)`, `external`, `blocks`, and `traps` … no
   region-bearing effect is admitted" [verified, spec:1183], so an
   `allocates(arena 'r)` row can never arrive through an indirect call.

**And a1's preferred repair costs nothing anyway, measured.** Across the entire
non-archive corpus, exactly **two** files declare `allocates(arena 'r)` on a
function boundary — `stor5-neg-arena-new-region-bearing.wf` and
`stor4-neg-arena-escape.wf` — **both negative conformance cases**; **zero** real
programs propagate an arena allocation across a boundary [measured,
`grep -rn "allocates(arena" tests/programs tests/conformance/cases research`].
So the "large expressiveness cost" a1 says the proposal hides behind "OPEN" is
currently **0**. That is an argument for choosing on principle rather than on
cost — and on principle, putting a non-observable into a *non-interference*
judgment is the wrong move: it would deny permission for a hazard the language
defines away.

**Refuted: the drop/release half.** a1 lists "the compiler-derived free/release
actions [STOR-3] that also run concurrently" as unaccounted. They are accounted,
or they are the heap: "A `box<T>` drop, a `buffer<T>` drop, an `arena<'r, T>`
region release, and the absent drop of a `const` item [CONST-2] each carry the
**empty** release row" [verified, spec:1400-1401]. Box and buffer frees are heap
allocator calls — §6's existing line, once corrected to name it. An arena's
region release happens at the region block's exit, i.e. **after** the join, by
STOR-4 confinement; it is never concurrent with a lane [derived]. The residue —
a compiler-owned resource family whose contract fixes a nonempty release row
[verified, spec:670-672] — carries `external`/`blocks` and is therefore already
excluded from overlap by the law.

**Amendment D (§3 and §6):**
> §3: *"`allocates(heap)` and `allocates(arena 'r)` do not enter P's footprint.
> Rationale: no observable in [PAR-0]'s tuple depends on allocator state —
> allocation-fit is discharged at compile time or the program is rejected
> [ERR-4], exhaustion is not a language trap [SCOPE-3], no operation yields an
> address as a value, and region storage is released only with its region
> [STOR-4]. Both allocators are TCB obligations, not permission obligations."*
>
> §6: *"The runtime guarantees, under concurrent invocation, every allocator the
> language derives: the heap allocator (`box_new`, `buffer_new`,
> `buffer_vacant`, and the compiler-derived frees of `box`/`buffer`) and each
> region's arena allocator (`arena_new`). These are accounted TCB lines."*

### 6b — peak memory — CONCEDE

The law's observable list does not enumerate it, spec:725 puts exhaustion
outside the language, and §2 relegates it to a bullet. a1 is right that the law
is incomplete as written. **And I must volunteer that Amendment A makes it
worse**: a lane parked behind a diverging earlier lane retains its stack
indefinitely, which is a real regression against R-a on this axis (g1 flagged
the same cost at `g1-divergence.md:534`).

**Amendment E (into the law text, not a bullet):**
> *"A conforming implementation bounds actualization width by an implementation
> constant fixed before execution. Peak live storage under overlap may exceed
> the sequential elision's by up to that factor. Resource exhaustion is not a
> language observable [SCOPE-3]; this rule therefore does not require overlap to
> preserve completion under a fixed memory budget, and an implementation that
> cannot honor its width bound must not actualize."*

That is honest rather than reassuring, which is the correct posture for a clause
a human approves.

### 6c — external aliasing — CONCEDE

**Amendment F (into the law text):**
> *"A body whose row carries `external` or `blocks` is never overlapped. This
> exclusion is a scope limitation of this version, not a discharged obligation:
> it carries both the EFF-5 ordering guarantee and the aliasing guarantee that
> two separately owned host handles may denote one host object, which P cannot
> see."*

---

## Amendment G — replacement falsifiers (§10)

Retired: "a compiling program that breaks P (footprints disjoint under OWN-7 yet
interfering)" — a1 probed for it and failed, and I did not attempt a rescue.

New falsifiers, all aimed at the amended framework:

1. **Lemma B falsifier.** A compiling program in which two P-permitted lanes'
   outcomes are *not* schedule-invariant. Any such program kills [PAR-2].
2. **Lemma A′ falsifier.** A §2 observable that depends on allocator state —
   e.g. a future capacity claim, an address-valued operation, or an arena with
   per-value release. Any one re-opens Amendment D.
3. **EFF-4 ruling.** If the owner reads "Trap is abort" as requiring immediate
   process termination, Amendment A dies and the framework falls back to R-b+
   (inferred `decreases` in ENT-6's fragment) with its full token cost and g1's
   measured 43/100 coverage collapse.
4. **Parked-lane memory.** A real program whose overlapped peak, with a lane
   parked behind a diverging sibling, exceeds an acceptable budget — this is
   Amendment A's genuine price and it is unmeasured.
5. **Retained from a1:** deep-fold peak-memory completion change (now
   dispositioned by Amendment E, still worth measuring).

## What this defense costs

- **Withdrawn:** R-a, R-b, R-c, §4's eligibility gate, §4's "no coordinator"
  phrase, §10's headline falsifier.
- **Added:** one spec clause [PAR-1] (~150 words) plus the [PAR-2] statement,
  three law-text sentences (Amendments E, F), and two corrected prose lines
  (§3, §6). **No checker work at all.** Argued: this is below R-b+'s token cost,
  which g1 priced as the largest of the four v0 options.
- **Unresolved and honestly open:** the parked-lane memory cost (falsifier 4);
  §9's items 3, 4, 5, 6, 8, 9, none of which a1 attacked and none of which this
  defense touches.
- **Owner boundary:** every amendment here is a `spec/kernel-spec.md` change.
  Nothing in this document is approved or authorized to land.

## Reproduction

```
whitefootc=<repository>/compiler/target/release/whitefootc
cd <the lead's scratch>/wf-parallelism-research/debate/probes

$whitefootc -o d1_two_traps.bin d1_two_traps.wf && ./d1_two_traps.bin   # left_small, exit 134
$whitefootc -o d1_one_site.bin d1_one_site.wf                          # fails at 150
$whitefootc -o d1_one_site_b.bin d1_one_site_b.wf                      # fails at 900
cmp d1_one_site.out d1_one_site_b.out                                  # identical
$whitefootc -o d1_gen_u64.bin d1_gen_u64.wf ; $whitefootc -o d1_gen_u32.bin d1_gen_u32.wf
cmp d1_gen_u64.out d1_gen_u32.out                                      # identical
$whitefootc -o d1_closure_div.bin d1_closure_div.wf ; timeout 5 ./d1_closure_div.bin  # exit 124
```

Reproduced from a1 independently: `a1_closure.wf`, `a1_two_sites.wf`,
`g1_siblings.wf` all compile (exit 0); `a1_arena_twosib.wf` stops at
`SemanticUnsupported { feature: ArenaRuntime }` after source acceptance.

Corpus measurements: `grep -rn "allocates(arena" tests/programs
tests/conformance/cases research` → 2 hits, both negative cases;
`tests/programs/*.wf` → 24 programs, 100 `fn`/`command fn` declarations, 53 with
`traps` in the row, 241 `claim` sites.

Nothing under the repository was modified.
