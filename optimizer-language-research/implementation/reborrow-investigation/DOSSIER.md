# No-reborrow investigation — dossier

Recorded investigation for the `THE-PLAN` Phase 2 "open decision — the no-reborrow
rule" gate. Date: 2026-07-18. Method: a read-only multi-agent investigation
(map the usage -> analyze each option -> adversarially attack each -> synthesize),
run against the real wfc source and against the parked-branch checker in a sandbox.
No repository file (spec, checker, tests, wfc source) was modified. This is a
recommendation for the owner, not a decision; a kernel-spec change remains
owner-gated (see `governance/APPROVALS.md`). Provenance: workflow wf_9da63778-857.

---

# Dossier: The No-Reborrow Rule (T-A) — Keep-and-Rewrite vs. Bounded Relaxation

Prepared for the owner. Read-only investigation; no repository file was modified. This is a recommendation, not a decision. The gated artifact at stake is a numbered kernel-spec rule (v0.6 OWN-1/4/6 + a possible new reference-result-provenance rule), so any change is owner-gated.

---

## 1. The finding and why it matters

**Finding.** Every reborrow in the production compiler (`compiler/src/*.wf`) is a transient, statement-scoped call argument that never escapes. Not one is bound to a name, stored in a struct, moved/`give`n, or returned. The compiler's entire dependence on reborrowing lives in the single easiest, best-guarded corner of the feature.

**Why it matters — the no-alias fact channel (F001).** The no-reborrow property (T-A, "singleton provenance") is what makes the optimizer's `noalias` base *unconditional*. Today the optimizer reasons: a live `&uniq 'a p` is the *only* usable mutable path to `p`, because a borrow's provenance is a singleton with no lineage — nothing was re-narrowed from it. That single sentence is the load-bearing premise. Relaxing the rule does not automatically break `noalias`, but it changes *why* `noalias` holds: from "no lineage exists" to "the root is singleton **and** every ancestor is suspended **and** every overlapping `uniq` sibling is rejected." That is a strictly more complex predicate the optimizer must be re-proved to trust. Per CLAUDE.md, a fact-channel change gets hostile adversarial review *before* shipping, and a green gate is not that review.

The owner's 2026-07-18 ruling frames the choice correctly: wfc is early code and is **not** a source of truth. The rule must be decided on soundness and cost, not on the convenience of making wfc pass.

---

## 2. Verified reborrow characterization

Confirmed against `compiler/src/*.wf` on `main`:

- **1,062 borrow-of-deref sites**: 989 `&uniq 'r deref(holder)` + 73 shared `&'r deref(holder)`. (The shared-context figures of ~1,206/1,279 over-count; the 989 enumeration is complete — holder sums close. This count discrepancy is **not** a soundness exception, but an owner-gated change should not cite a moving number as its evidence.)
- **Holder distribution (uniq)**: out 747, report 82, facts 61, symbols 39, scratch 20, types 15, validation 12, analyzed 11, tokens 1, ast 1. `out` is the byte-sink; the rest are struct-of-arrays tapes.
- **Every holder is a `&uniq` (or borrow) *parameter*, never an owned local.** The owned locals of the same names exist only in the constructor `frontend_unit_new`, are accessed by `move`, and are then sealed into `own AnalyzedUnit`. `deref` requires a reference, so `deref(owned_local)` neither occurs nor is possible.
- **No escape on any channel** (each verified):
  - No reborrow is bound to a borrow-typed variable (`grep '= &(uniq|shared).*deref('` → nothing).
  - No struct field is borrow-typed anywhere (`field: &...` → zero). Structs hold values (PATTERNS).
  - Every function returns `own …` (488 `-> own` heads; zero return `&uniq`/`&shared`/`&'r`).
  - `give` appears nowhere in `compiler/src/*.wf`.
- **Canonical shape** (`frontend.wf:451-471`, `frontend_unit_reset`): each reborrow sits alone in its own single-statement `region '…` block; the child is created, passed, consumed, and dropped inside one statement while the parent is textually inaccessible except through the borrow — exactly the OWN-5 holder-only exception.

**Bottom line for §2:** wfc uses only the statement-scoped, non-escaping *fragment* of reborrowing. It exercises none of the hard cases (bound children, result-transfer, downgrade, loan-after-holder-move).

---

## 3. Option A — Keep no-reborrow, rewrite wfc

**Verdict: viable for ~979 sites; the ~10-site analyzer core has no acceptable-cost rewrite.**

### Pros
- **Zero new soundness obligation.** Keeping T-A preserves unconditional singleton provenance; F001 needs no re-derivation. This is the only path that opens nothing.
- **Two of three tiers rewrite cleanly and *eliminate* the reborrow (not relocate it):**
  - *Trivial tier (8 sites, e.g. `frontend_unit_reset`):* field writes through the holder are the OWN-5 exception, not a borrow. The six reborrows collapse to `set deref(analyzed).tokens.count = 0_u64; …`. Zero new borrows, zero perf cost.
  - *Byte-sink tier (119 of 120 `out: &uniq ByteTape` functions):* return the borrow — `-> &uniq 'o ByteTape` — and thread it linearly (`set out = byte_tape_push(out: move out, byte: b)`). The borrow is *moved*, never forked (OWN-4: return region = param region). Zero-cost (a `&uniq` is a pointer; move is a register copy; the buffer stays put).

### Cons / where it fails
- **The value-returning, multi-borrow analyzer core has no blessed-pattern rewrite.** `semantic_body_analyze_atom` returns `own u64` (a type id) *and* mutates two `&uniq` borrows (`facts`, `report`), keeping both live across two recursions plus a post-join write. 10 of 31 `facts`-taking functions are value-returning and hit this wall; they are the hottest, most-recursive code in the compiler.
- **`take`/`replace` / move-field-out is dead for all 989 sites.** Every holder is a `&uniq` *parameter*; OWN-5 forbids moving content reached through a borrow, and STOR-1 marks in-place buffer take/replace unresolved.
- **By-value threading of `ByteTape` is structurally blocked** — the buffer is owned by the C/FFI caller; wfc only ever holds `&uniq` and can never move out of it. Byte-sink is forced onto borrow-return, a **novel idiom not in PATTERNS P1-P9**.
- **Reuse loss and verbosity.** `semantic_type_tape_reset`/`semantic_node_facts_reset` are called from both `frontend.wf` and `semantic_unit.wf`; specialize-to-aggregate duplicates helpers; inline-through-holder duplicates bodies with drift risk. ~120 byte-sink signatures + all call sites change.

### Did the attack show rewriting is infeasible or merely costly?
The adversarial attack targeted the decisive tier — the mutually-recursive analyzer (`semantic_body.wf:718` signature, `:973`/`:998` twin reborrowed recursions, `:1021` post-join `set deref(facts).type_ids[node] = bool_type`). Five encodings were tried:

1. **Linear-thread the borrow** — dead. A `&uniq` is affine; to keep `facts` across two recursions + the join-write the child must *return* it, but the function must also return `own u64`. wf has single-return, zero borrow-returning functions, and no borrow-typed struct fields (all verified), so the needed `(u64, &uniq, &uniq)` co-return is unrepresentable.
2. **Pass the whole aggregate as `&uniq`** — dead: `analyze(view: &uniq 'child deref(view))` **is itself a reborrow**. It renames what gets reborrowed at the identical site; it does not even push it into the callee.
3. **Owned mega-context threaded by value with an in-band result slot** — the only mechanically reborrow-free encoding, but **its owned root does not exist anywhere in the analysis chain.** `NodeFacts`/`TypeTape` are `own` for one statement in `frontend_unit_new`, then sealed into `own AnalyzedUnit`; every pass sees them only as `&uniq deref(...)`. `SemanticBodyScratch` is never owned at all. Constructing the owned context requires whole-pipeline ownership inversion (thread `own AnalyzedUnit` by value front-to-back) **or** the STOR-1-blocked struct-field take/replace — and the shape it produces is precisely the FN-7 opaque per-call-tree mutable context (result smuggled through a field, not the signature) that erodes the very field-disjointness F001 protects.
4. **`split_uniq` disjoint-field view** — useless: the recursion writes `type_ids[left_node]` while the parent writes `type_ids[node]` on the *same* array at node-id indices that are parse-order, not statically disjoint.
5. **Restructure so each recursion is last-use** — impossible: `facts` genuinely survives left recursion, right recursion, and a use after both.

**Attack conclusion (`broke_it: true`):** Option A's claim that all 989 sites rewrite "at acceptable cost" is broken *for the analyzer core*. The break rests on the task's "or only at unacceptable cost" clause, not strict impossibility — Encoding 3 is technically a reborrow-free program, so a determined rearchitect could redesign the analyzer around a single top-down owned semantic context (a large but *finite* rewrite). **Marked thin:** the perf magnitude of the large-header-move cost was reasoned structurally, not measured (read-only task); and no single numbered rule states "no field may be borrow-typed" — it was confirmed empirically (zero in `*.wf`) and via PATTERNS + TYPE-2, not from a cited rule.

---

## 4. Option B — Relax to bounded, non-escaping, statement-scoped reborrows

Child-suspension model as drafted on branch `parked_edits` (OWN-6 rewritten + new reference-result provenance; checker in `prototype/checker/checker.py`).

**Verdict: viable-with-cost; survived a genuine kill-shot attempt but rests on an OPEN formal obligation (OBL-4) and an un-run fact-channel audit.**

### Pros
- **The single-writer invariant is preserved for wfc's actual shape.** F001's real claim is "at most one *usable* mutable path per place," not "no parent-child lineage." Under child-suspension, authority moves *down* the ownership tree (child's resolved place = parent's + a suffix) and never forks: before the reborrow the sole path is the parent; during it the sole path is the child and the parent is textually inaccessible; after, the parent resumes. Pointwise, two usable mutable paths never overlap.
- **The wfc-necessary subset is the trivial fragment** — statement-scoped suspension only (OBL-4 item 1). It uses none of the hard machinery (no bound children, no result-transfer, no downgrade). Deciding it is syntactic: recognize `&uniq 'c deref(h)[suffix]`, set a suspend flag on `h` for the statement, reuse the existing OWN-5/OWN-7 overlap check for siblings. No dataflow fixpoint.
- **Singleton provenance (T-A) is retained, not discarded.** A live borrow still has one immutable resolved root; lineage branches but never retargets, merges, or reassigns. FR's lval-set/path-set retyping stays collapsed to singletons.
- **Formal antecedent exists** — this is FR's own `*w` reborrow restricted to singleton-rooted child lineages, so there is something to reconcile against rather than a novel invention. The parked branch has 167 checker tests + policy-oracle regeneration.
- **Unblocks Phase-2 self-hosting at the source level with zero rewrite** of ~1,000 sites, and dodges the STOR-1 take/replace hazard Option A repeatedly hits.

### Cons
- **OBL-4 is OPEN.** The parked branch's headline "10,000-model, zero violations" run is explicitly an *existing-core* run — it validates the pre-reborrow checker. The new forms (suspension/resumption, loan-after-move, result-transfer, whole-arg roots, downgrade) have only focused unit regressions. The branch itself marks the additive forms **provisional**.
- **Fact-channel review not done.** This is a fact-channel change; CLAUDE.md mandates hostile review before shipping. The `noalias` *derivation* must be re-audited under lineage, and 167 tests + a green make are the green gate, not that review.
- **Checker complexity is real** because the *spec* admits the full rule, not just wfc's subset: `checker.py` +804/-54, 14 new helpers (provenance fingerprints at flow joins, moved-holder loan persistence, descendant-mode propagation, reference-result provenance recovery/downgrade). The simple case cannot be admitted in isolation without speccing the bound/returned/downgraded cases too.
- **The measured performance case for B is absent.** The binary-trees port hit the no-reborrow wall and was forced to a bottom-up SoA build at **zero measured cost** (0.71s with facts vs. an inexpressible Rust recursive-&mut arena; identical-shape Rust 0.64s). So B buys *writer ergonomics / avoiding a wfc rewrite*, not speed.
- **Reborrow-adjacent escape is a demonstrated FATAL-race class.** D18-R20..R24 found an interior `&uniq` view whose result region was caller-supplied — two live `&uniq` to one location. Sequential wfc reborrows don't escape, but the *spec rule* permits result-carrying children, the same shape that raced; its escape-prevention clauses (OWN-4/OWN-10/reference-result provenance) are load-bearing and must be attacked directly.
- **Governance:** the parked branch edited v0.6 in place, which GOV-1/spec-guard now hard-fails. Any real change must bump the version and rename the file.

### Soundness verdict — did the kill-shot break it?

**No kill-shot found (`broke_it: false`).** A live sandbox was built from the parked-branch toolchain (`democ.py` + the drafted `checker.py`) and 20 programs were driven through the exact checker. Every task-suggested aliasing vector was defused by a specific, working clause:

- Sibling `uniq` reborrows of overlapping fields → OWN-5 sibling-overlap bar (`neg1`, `probe3`).
- A child whose region outlives the parent's suspension → OWN-4 forces result region ≤ source-argument region; parent resumes only after its **last** descendant leaves the borrow set (`escape1/2`, `resume1`).
- `uniq` child from a `shared` parent → rejected ("uniq borrow through shared reference").
- Reborrow × move/downgrade/effects → loan persists after holder move (`move1-3`); region-granular `writes()` check fails closed (`probe6`); downgrade retains a shared loan that blocks a fresh `uniq` (`probe9/10`).
- Nested/transitive reborrows → transitive `derived_from` suspension (`probe7`).
- The canonical wfc shape → **ACCEPT** (`wfc`), confirming B unblocks the ~1,000 sites.

The **closest approach** was the sequential analog of the D18 guard-FATAL (`escape1/escape2`): an interior `&uniq` view escaping past its object's tenure via a caller-supplied result region. In concurrency this raced because a mutex's dynamic tenure was shorter than the caller region. In the *sequential* kernel that mismatch cannot exist — reference-result provenance requires the formal return region to be syntactically identical to a formal *parameter* region, so the result is tied to its source, and OWN-4 rejects any borrow whose region does not outlive the destination. Both escapes were caught with the OWN-4 diagnostic.

**What the rule must forbid (and, as drafted, does):** (1) two overlapping `uniq` siblings on one call; (2) a `uniq` child from a `shared` parent; (3) any access — read/write/move/copy/transfer — through a *suspended* parent; (4) a child whose region outlives its source (return/store/`give` past the parent); (5) resumption before the *last* descendant ends. All five are named and enforced.

**Honest residual risk (this is a failure-to-break, not a proof of soundness):**
- **OBL-4 is still open.** 20 hand-built programs + a code read are weaker than the bounded 10k-model check, and the parked 10k run covered only the pre-reborrow core. Bounded model checking over the *widened* form space is exactly what would catch a shape neither the drafters nor the attacker imagined.
- **Safety is enforced as distributed runtime bookkeeping** (a `derived_from` back-pointer per borrow + timely removal from the borrow set), not as a structural invariant. A future edit that mints/transfers a child without threading `derived_from`, or drops a borrow one region-exit early, silently reopens aliasing **with a green gate**. Recommend encoding the suspension invariant structurally, or adding a checker-internal assertion that no two live `uniq` borrows have overlapping resolved places unless one `derives_from` the other.
- **The attack hit the checker, not the optimizer's `noalias` derivation.** That fact-channel re-audit is still owed.
- **Slices** are a latent shared/uniq place-aliasing surface, orthogonal today (`slice_of` is shared-only; democ lacks slice deref) but needing review if uniq slice views or affine-element buffers (both DEFERRED) are ever added.

---

## 5. Comparison

| Dimension | Option A — keep + rewrite | Option B — bounded relax |
|---|---|---|
| New soundness obligation | **None** (T-A unchanged) | **OBL-4 open**; `noalias` re-derivation owed |
| F001 / `noalias` | Unconditional, untouched | Preserved *if* re-derived under lineage (unaudited) |
| Kill-shot result | Rewrite claim **broke** on analyzer core | **No kill-shot** across 20 programs |
| wfc coverage | ~979/989 sites clean; **~10 hottest sites have no acceptable-cost rewrite** | 100% of sites, zero rewrite |
| Blessed-pattern fit | Trivial + byte-sink OK; byte-sink needs a **new** borrow-return idiom; analyzer core → FN-7 shape (unblessed) | New numbered rule(s); statement-scoped subset is simple |
| Perf | Zero-cost for the two easy tiers; mega-context = large-value moves (unmeasured) | Zero source rewrite; no measured perf *gain* (binary-trees SoA was already free) |
| Cost locus | Front-loaded, mechanical, but architectural wall on hottest recursive code | Spec mass (91 rules; OWN-1/4/6 3× prose), checker +804/-54, OBL-4 + fact-channel review |
| Governance | None (no spec change) | Owner-gated: version bump + file rename + reconciliation + hostile landing |
| Residual risk | Analyzer core may force whole-pipeline rearchitecture (finite but large) | Failure-to-break, not proof; distributed-bookkeeping fragility |

---

## 6. Recommendation

**Evidence is insufficient to ratify Option B as drafted today — but the direction it points (a *minimal* statement-scoped relaxation) is better-founded than pure Option A, and pure Option A is not clean either.** Concretely:

1. **Do not adopt pure Option A.** The attack broke its "acceptable cost" claim on the analyzer core: the only reborrow-free encoding for the hottest, most-recursive code (`semantic_body_analyze_atom` and its family) is the FN-7 opaque owned-mega-context that no pattern blesses and that erodes the very F001 disjointness T-A protects — or a large whole-pipeline ownership inversion colliding with STOR-1. Keeping T-A is free *only* for the trivial and byte-sink tiers (≈979 sites); it is architecturally expensive exactly where the compiler is hottest.

2. **Do not ratify Option B as drafted.** It survived a real kill-shot attempt, which is meaningful, but (a) OBL-4's widened bounded model check has **not** been run for the new forms, (b) the `noalias` fact-channel re-derivation under lineage has **not** been audited, and (c) safety is carried as distributed runtime bookkeeping rather than a structural invariant. CLAUDE.md forbids shipping a fact channel on a green gate alone.

3. **Recommended path: pursue the narrowest relaxation that covers 100% of wfc — OWN-6 statement-scoped suspension only — gated on three concrete deliverables.** wfc needs *only* OBL-4 item 1 (statement-scoped suspension of an unbound call-argument child); it uses none of reference-result provenance's result-transfer, downgrade, or loan-after-move. A minimal rule that admits *only* the unbound statement-scoped fragment carries a much smaller obligation and a fraction of the checker code, and still unblocks every wfc site. Defer reference-result provenance's result-carrying/downgrade machinery until an actual writer need for it is demonstrated (none exists today).

**This recommendation is explicitly not "relax because wfc uses reborrows."** wfc's usage is treated as a *cost signal for Option A*, not a justification for B. The decisive facts are: (i) Option A's rewrite is architecturally unacceptable on the analyzer core, and (ii) Option B's minimal fragment preserves the single-writer invariant and survived adversarial attack — subject to unfinished formal work.

**What would change this recommendation:**
- **Toward pure Option A (keep, no relax):** if a measured redesign shows the analyzer core can move to a single top-down owned semantic context at tolerable perf and readability cost (the finite-but-large rewrite), *and* the byte-sink borrow-return idiom is blessed and measured zero-cost — then keep T-A and pay the one-time sweep. wfc is early code; a bounded rewrite there is cheaper than permanently widening the language.
- **Toward full Option B:** if a writer need for *bound/returned* reborrows (reference-result provenance forms) is demonstrated beyond the statement-scoped fragment, and OBL-4 is discharged for those forms too.
- **Toward a third option not costed here:** a checked `split_uniq` disjoint-field view (already carded in PATTERNS "Known gaps") could cover the field-projection subset (`frontend_unit_reset`, `deref(out).bytes`) *without* general reborrow, keeping T-A intact for whole-aggregate re-narrowing. It does **not** help the analyzer core (same array, node-id indices, not statically disjoint), so it is a partial mitigation, not a full answer — but it is worth costing.

---

## 7. Next steps if the owner chooses to proceed (with a relaxation)

A numbered-kernel-spec change is owner-gated. Before any edit, the owner must be notified with the exact proposed delta and give explicit approval; approval of this dossier is not approval to edit the spec.

**A. Governance / spec mechanics (mandatory, mechanical):**
- Bump the specification version and **rename the file** (`kernel-spec-v0.6.md` → next version); update title and every live reference in the same change. Never revise a numbered rule in place — the parked branch violated this and GOV-1/spec-guard now hard-fails it.
- Scope the delta to the **minimal** rule: OWN-6 statement-scoped suspension for an unbound call-argument child. Explicitly exclude (or separately gate) reference-result provenance result-transfer, uniq→shared downgrade, and loan-after-holder-move until needed.
- Keep `CLAUDE.md`/`AGENTS.md` byte-identical if either is touched.

**B. Formal reconciliation (discharge OBL-4 for the admitted fragment):**
- Run the bounded model check on the **widened** form space (extend the existing 10k-model harness with the child-suspension form), and record it — the existing clean run is pre-reborrow-core only and does not count.
- Complete the M0/FR reconciliation for the admitted fragment: reconcile against FR's `*w` reborrow restricted to singleton-rooted child lineages, showing lval-set/path-set retyping stays collapsed to singletons.
- Pin one authoritative, occurrence-aware census of the reborrow sites (the 989 `&uniq 'r deref(ident)` enumeration is complete) so the evidence cited is not a moving number, per the strengthened GOV-2 evidence rule.

**C. Hostile landing (the fact-channel review a green gate does not substitute for):**
- **Re-audit the optimizer's `noalias` derivation under lineage:** prove `noalias(child)` follows from "singleton root ∧ every ancestor suspended ∧ every overlapping `uniq` sibling rejected," not from the retired "no lineage" premise. This is the CLAUDE.md-mandated fact-channel review and it is currently un-run.
- **Attack the escape-prevention clauses directly in the sequential setting** (do not assume them safe by analogy) — specifically the D18-R20..R24 guard-FATAL shape (result-carrying child escaping past its source region); the drafted OWN-4 tie caught it in sandbox, but it must be attacked as a spec rule, not just observed passing.
- **Harden the enforcement mechanism:** encode the suspension invariant *structurally*, or add a checker-internal assertion that no two live `uniq` borrows have overlapping resolved places unless one `derives_from` the other — so a future edit that forgets to thread `derived_from` fails loudly instead of silently reopening aliasing behind a green gate.
- **Flag slices for a follow-up review** if uniq slice views or affine-element buffers (both DEFERRED) are later added — a slice reroots places off the buffer and would defeat the OWN-7 overlap check every rejection above depends on.

**Durability:** land as a single change with a `decision-gates.md` line recording the ruling and the evidence, so the decision survives session rewind.

---

*Evidence-quality flags: the reborrow characterization (§2) and the Option-B sandbox result (§4) are directly verified against source / the parked-branch checker. Marked thin and not re-derived by running the toolchain: the perf magnitude of Option A's mega-context moves (reasoned structurally, unmeasured); the illegality (vs. mere absence) of borrow-typed struct fields (confirmed empirically + PATTERNS/TYPE-2, no single cited rule); and whether OBL-4's widened model check would surface an un-imagined aliasing shape (the open obligation itself).*

---

## Appendix — raw adversarial verdicts

- **Map (usage).** All reborrows are transient, statement-scoped, non-escaping call
  arguments through `&uniq` parameters; zero are bound, returned, stored, or given.
  Exceptions found: 0. Authoritative count: 989 `&uniq` + 73 shared
  = 1,062 sites (the ~1,206/1,279 figures over-count).
- **Option A attack — broke_it = TRUE.** The recursive analyzer core
  (`semantic_body_analyze_atom` family) has no acceptable-cost no-reborrow encoding; the
  only reborrow-free shape is an FN-7 opaque owned mega-context or a STOR-1-blocked
  take/replace.
- **Option B attack — broke_it = FALSE.** No kill-shot across 20
  programs driven through the parked-branch checker; the closest approach (the sequential
  analog of the D18-R20..R24 guard FATAL) was caught by the OWN-4 region tie. This is a
  failure-to-break, not a proof of soundness; OBL-4 and the noalias fact-channel review
  remain open.
