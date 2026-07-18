# Minimal reborrow relaxation — proposed rule (DRAFT rev 2, not yet a spec change)

Status: DRAFT proposal for the narrow-relaxation direction (owner-selected 2026-07-18).
This is the anchor artifact the model-check, the Featherweight-Rust (FR) reconciliation,
and the hostile no-alias review verify against. It is NOT a spec edit; the kernel spec is
owner-gated and changes only after the evidence is in and the owner approves the exact
delta (see `governance/APPROVALS.md`).

**Rev 2 (2026-07-18)** applies the fixes from the adversarial soundness/coverage/
decidability review. Rev 1 was BROKEN: it asserted the child "cannot escape because
functions return `own` only" — false for v0.6, which permits borrow-returning functions, so
a callee could launder the child's loan into a bound result that outlives the statement
(two usable `&uniq` to one place → F001 miscompile). Rev 2 closes this with admission
condition 4 (receiving call returns a non-borrow) plus `'c`-confinement, and fixes the
OWN-7 index-sibling gloss and the OWN-5/OWN-12/ancestor interactions. Coverage stays 100%.

Design intent: admit exactly the fragment of reborrowing that wfc uses — a transient,
non-escaping child borrow passed as a call argument to an `own`-returning callee — and
nothing more. Every harder form (bound children, returned/given children, result-carrying
children/reference-result provenance, uniq→shared downgrade, loan-after-holder-move) is DEFERRED, so the open
obligation and the checker surface stay small. See `DOSSIER.md` for why this fragment is
the recommendation.

## 1. Scope — what is admitted

A **statement-scoped child reborrow** is the written expression

    &uniq 'c deref(h)[suffix]        (uniq child)
    &'c   deref(h)[suffix]           (shared child)

appearing as an **argument atom of a `call` expression** [GRAM-9], where `h` is an eligible
live borrow holder (§2), `'c` is a locally-introduced region confined to the enclosing
statement (§5), and `[suffix]` is a possibly-empty place projection.

Admitted iff ALL hold:

1. **Unbound call argument.** The reborrow expression is itself an argument atom of a call;
   it is NOT the entire initializer of a `let`/`set`, a `return`/`give` operand, a
   construct field value, a `match` scrutinee, or an `index` offset. It MAY be nested as an
   argument to a call that itself occupies a `let`/`set`/`return` position — e.g.
   `set x = f(..., facts: &uniq 'c deref(facts), ...)` is admitted (the reborrow is an
   argument to `f`; `x` binds `f`'s `own` result, not the reborrow). GRAM-9's no-nested-call
   form means a child argument cannot transitively feed another call.
2. **Receiving call returns a non-borrow.** The call the reborrow is an argument to has
   result mode `own` (or `unit`) — never a borrow. This forbids a callee from laundering the
   child's loan out through its return value. (Verified: every wfc callee returns `own`, so
   this keeps 100% coverage.)
3. **Uniq requires uniq.** A `uniq` child requires a `uniq` parent (`h` is `&uniq`). A
   shared child is allowed from either.
4. **Sibling compatibility (defer to OWN-7).** Children created in the same statement are
   judged by OWN-5/OWN-7 on resolved places: any overlapping pair containing a `uniq` child
   is rejected; shared siblings may overlap. Two `index` places over the same resolved base
   are disjoint iff **both indices are literals with unequal values**; any pair with a
   non-literal index, or equal literals, overlaps. Distinct fields are disjoint.

## 2. Eligible holder and suspension

**Eligible `h`.** `h` is a borrow holder that is a function **parameter** or a `let`-bound
borrow of a place — never a `match` binder and never itself a statement-scoped child
(children are unbound per §1.1). Consequently an eligible `h` has no reborrow ancestor, so
the transitive-ancestor machinery below is vacuous in this fragment; `match`-binder parents
and grandchild chains are DEFERRED (§6).

**Parent/ancestor.** `parent(child)` is the holder named in its `deref(h)`; `ancestors` is
the transitive `derived_from` closure. In the admitted fragment this closure is `{h}`.

**Suspension.** Creating a child reborrow of `h` **suspends** `h` for the enclosing
statement. While `h` is suspended:

- OWN-5's holder read/write allowance for `h` is itself suspended: no read, write, move,
  copy, or call-transfer through `h` is permitted;
- the sole permitted operation naming a place overlapping `resolved(h)` is creating a
  further sibling child in the same statement (judged per §1.4);
- `h` **resumes** at the end of the enclosing statement, after its last child's borrow
  ends. There is no earlier resumption: the child cannot be bound (§1.1) and its `'c` cannot
  outlive the statement (§5), so no child persists past statement end.

`resolved(child) = resolved(h) ++ suffix`. All OWN-5/OWN-7 judgments use resolved places.
Singleton provenance (T-A) is retained: the child has one immutable resolved root; lineage
branches but is never retargeted, merged, or reassigned.

## 3. What is forbidden — and why each is load-bearing for `noalias`

The no-alias fact F001 requires: **at most one usable mutable path to any place at any
instant.** Each forbiddance closes a way that invariant could break.

| Forbidden | Rule that rejects it | Why it would break `noalias` |
|---|---|---|
| Two overlapping `uniq` siblings | §1.4 (OWN-7) | Two usable `&uniq` to the same place at once |
| `uniq` child from a `shared` parent | §1.3 | A shared alias would coexist with a fresh `uniq` write path |
| Any access through a suspended parent | §2 suspension (incl. OWN-5 allowance suspended) | Parent + child = two usable mutable paths |
| Child laundered out via a borrow-returning callee | §1.2 (rmode non-borrow) | A returned borrow rehomes the child's loan into a binding that outlives the statement |
| Child bound / returned / given / stored / in a caller-supplied `'c` | §1.1 + §5 `'c`-confinement | Child would outlive the parent's suspension |
| Resumption before the last child ends | §2 resume-at-statement-end | Parent usable while a child path is live |

Escape-freedom is established structurally, NOT by "functions return `own` only" (that is a
wfc-empirical fact, not a language invariant). It rests on: the receiving call returns a
non-borrow (§1.2), so no loan derived from the child leaves via a result; the child is an
unbound argument (§1.1) and structs hold no borrows [see §8 note on the numbered rule];
`'c` is statement-confined (§5). A borrow can leave a callee only via its return value, and
§1.2 forbids that value from being a borrow.

## 4. `noalias` re-derivation (the fact-channel claim to be reviewed)

Old premise (v0.6): `noalias(x)` because `x`'s provenance is a singleton **with no
lineage**.

New premise (this rule): `noalias(child)` because (a) the root is singleton, (b) every
ancestor is suspended (no usable path through them), (c) every overlapping `uniq` sibling is
rejected, **and (d) no borrow derived from the child outlives its enclosing statement**
(enforced by §1.2 rmode-non-borrow and §5 `'c`-confinement). During the child's life the
child is the sole usable mutable path to its resolved place; at statement end the child is
gone, no derived borrow survives (conjunct d), and the parent resumes as the sole path.
Authority moves *down* the ownership tree and back up, never forks.

Conjunct (d) is the one rev 1 omitted and that the miscompile exploited. This re-derivation
is a strictly more complex predicate than the old one and MUST pass the hostile fact-channel
review (step 3 of the program) before any spec change. A green checker is not that review.

## 5. Decidability / checker cost

The admitted fragment is syntactic and local:

- recognize the written form `&uniq 'c deref(h)[suffix]` / `&'c deref(h)[suffix]` as a call
  argument atom [GRAM-9];
- check the receiving call's result mode is non-borrow (§1.2);
- require `'c` to be a **locally-introduced** `region 'c { … }` whose block does not extend
  beyond the reborrow's enclosing statement — reject a caller-supplied region parameter or
  any `'c` outliving the statement (`'c`-confinement, closes the inter-procedural escape);
- set a suspend flag on `h` for the statement; reuse the existing OWN-5/OWN-7 resolved-place
  overlap check for sibling compatibility;
- **OWN-12 effect rows:** when checking the receiving call's instantiated effect row
  (`writes('c)` / `reads('c)`) under OWN-5, exclude `h` (the child's ancestor loan) from the
  overlap test via the same `derived_from` exemption used for sibling overlap; every
  non-ancestor live borrow is still checked (otherwise every admitted reborrow false-rejects,
  or a genuine non-ancestor conflict is silently dropped);
- clear the flag at statement end.

No dataflow fixpoint, no path-set typing, no inference. Far smaller than the parked branch's
full design (reference-result provenance result-transfer, downgrade, loan-after-move — all deferred here).

## 6. Deferred (re-entry triggers recorded)

Each returns for its own owner-gated review when a real writer need is demonstrated:

- **Bound children** (`let c = &uniq 'r deref(h)...`) — needs child liveness beyond one
  statement.
- **Result-carrying children / reference-result provenance** (the parked branch's reference-result-provenance rule) —
  a function returning a borrow derived from a borrow argument. wfc uses zero of these; §1.2
  forbids them.
- **`match`-binder parents and grandchild reborrow chains** — needs the transitive-ancestor
  suspension and OWN-13 composition; vacuous in this fragment.
- **uniq→shared downgrade** and **loan-after-holder-move** — the parked branch's harder
  clauses; wfc exercises none.

## 7. Coverage discharge

Exhaustive scan of all 32 `compiler/src/*.wf` (adversarial coverage review, 2026-07-18):
989 `&uniq 'r deref(h)` + 73 shared `&'r deref(h)` = **1,062 sites, 100% admitted**, zero
rejected. All are unbound call arguments to `own`-returning callees, `deref(<bare-holder>)`
heads over ten holder classes (out, report, facts, symbols, scratch, types, validation, ast,
analyzed, tokens), single-field or empty suffixes, each in a fresh single-statement
`region '<name> { <one call> }` block. The §1.2 rmode-non-borrow fix does not reduce
coverage. Canonical same-holder disjoint-field sibling site: `frontend.wf:707`. This is the
pinned census evidence (GOV-2); the 1,206/1,279 figures over-count.

## 8. Obligation discharge plan and eventual spec delta

**Discharge plan:**
- **Model-check (OBL-4 item 1):** extend the soundness model-checker to GENERATE the
  statement-scoped child-reborrow form AND the two escape vectors this review surfaced —
  (a) borrow-returning callees, (b) caller-supplied and multi-statement `'c` regions — and
  assert no accepted program has two live `uniq` borrows with overlapping resolved places
  unless one derives from the other. The existing 10k-model clean run covered the pre-reborrow
  core only.
- **FR reconciliation:** reconcile against Featherweight Rust's `*w` reborrow restricted to
  singleton-rooted child lineages (checker stays singleton-rooted, no lval-set retyping).
- **Hostile fact-channel review:** re-derive `noalias` per §4 (all four conjuncts); attack
  the §3 escape clauses directly in the sequential setting; recommend a checker-internal
  assertion that no two live `uniq` borrows have overlapping resolved places unless one
  derives from the other.

**Eventual spec delta (owner-gated; drafted only, not applied).** Bump `kernel-spec-v0.6.md`
→ `v0.7`, rename the file, update title and all live references in one change. The exact,
review-conditioned delta and the binding conditions A–F are in `PACKET.md` (the owner-approval
artifact). In summary: OWN-5 carve-in/carve-out realized as a structural SUSPENDED flag;
OWN-6 statement-scoped child definition + eligible-holder restriction + suspension/resumption;
OWN-12 ancestor-only effect-row exemption; PATTERNS P4 no-reborrow → bounded reborrow; and —
correcting rev 2 — **one NEW numbered rule** promoting "no field/element/box-or-arena
content/enum-payload may be borrow- or region-typed" from convention to cited spec (the
own-wrapped-launder closure the fact-channel review surfaced). reference-result provenance result-transfer stays
deferred. Record the approval and evidence pointer in `governance/APPROVALS.md` and
`decision-gates.md`.

**Rev 3 (2026-07-18):** the FR reconciliation + hostile no-alias fact-channel review returned
PASS-WITH-CONDITIONS (no attack broke `noalias`; performance preserved; FR-singleton-rooted for
this fragment). It corrected rev 2's "no new numbered rule" plan (Condition A above) and
requires the exclusivity assertion to ship UNCONDITIONALLY (no `derived_from` carve-out) with a
lineage-aware alias-scope emitter. See `PACKET.md` for the binding conditions and the exact
delta; that is the authoritative artifact for the owner-gated decision.
