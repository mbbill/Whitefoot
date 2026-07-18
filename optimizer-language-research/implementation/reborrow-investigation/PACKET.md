# Owner-approval evidence packet — bounded reborrow relaxation (v0.6 → v0.7)

Date: 2026-07-18. This packet assembles the evidence for the owner-gated decision to relax
the no-reborrow rule to admit the minimal bounded fragment. **It is not a spec change.** Per
`governance/APPROVALS.md`, the kernel spec changes only after the owner approves the exact
delta below; approving a direction is not approving a spec edit. No spec/checker/test file is
modified by this packet.

## 1. Decision requested

Approve (or decline) the exact `v0.6 → v0.7` delta in §5, which admits **statement-scoped,
non-escaping child reborrows** (a temporary `&uniq`/`&` of a sub-place of an exclusive
holder, passed as a call argument), and defers all harder reborrow forms. Approval authorizes
the gated spec change and the production checker/wfc/conformance work that follows.

## 2. Why (one paragraph)

wfc's own source needs reborrows in ~1,062 places, concentrated in its hottest recursive
code, and the investigation (`DOSSIER.md`) showed keeping the rule forces those functions
into an anti-pattern (an opaque owned mega-context) that erodes the very no-alias facts the
rule protects. Relaxing to the *bounded* fragment removes that pressure **without losing
performance**: the optimizer still gets `noalias` on every exclusive borrow, because while a
child is live its parent is suspended and overlapping exclusive siblings are rejected — one
usable write-path per place, always. This is not "relax because wfc does it": wfc's usage is
the cost signal; the decision rests on the soundness and cost evidence below.

## 3. Evidence (four independent legs, all recorded)

| Leg | Result | Artifact |
|---|---|---|
| **Coverage / map** | Every wfc reborrow is a transient, non-escaping, statement-scoped call argument to an `own`-returning callee; 989 uniq + 73 shared = **1,062 sites, 100% admitted**, 0 rejected. Census pinned. | `DOSSIER.md` §2, `MINIMAL-RULE.md` §7 |
| **Model-check** | **0 soundness violations over 1,000,000 programs**; 100% of 363,153 launder attempts rejected; oracle proven non-vacuous. Discharges OBL-4 item 1 at the model level. | `modelcheck/RESULTS.md` |
| **FR reconciliation** | **HOLDS** — the rule is Featherweight Rust's `*w` reborrow restricted to singleton lineages; checker state stays a set of singleton-rooted loans (no lval-set retyping); Whitefoot's lexical suspend is **stricter-sound** (a subset of FR's flow prohibition). For the vacuous-ancestor fragment. | review record |
| **Hostile no-alias fact-channel review** | **PASS-WITH-CONDITIONS** — all three attack lenses returned `noalias` unbroken; no reachable miscompile in v0.6. Surfaced conditions A–F (§4) the delta must carry. | review record |

## 4. Binding conditions the v0.7 delta MUST carry

These came out of the fact-channel review. Conditions A and B are **NO-GO blockers** — the
spec edit must not be applied without them; they are already written into the §5 delta.

- **A. New numbered borrow-free-storage rule.** Promote "no struct field, array/buffer
  element, `box`/`arena` content, or enum payload may be borrow-typed or region-typed" from
  an un-numbered convention to a **cited numbered rule**. This is what makes the child truly
  unable to escape (it closes the *own-wrapped launder*: a callee returning `own Wrapper` that
  stashes the child borrow in a field — legal under a mode-only check, impossible under this
  rule). **This corrects rev 2's "no new numbered rule" plan.**
- **B. Structural suspension + unconditional exclusivity assertion.** Realize parent
  suspension as a lexical statement-scoped SUSPENDED flag that removes the parent from the set
  the exclusivity check consults, so the check needs **no `derived_from` carve-out**. Ship the
  invariant unconditionally: *no two live usable `uniq` borrows have overlapping resolved
  places* (the "unless one derives from the other" exception is deleted — it would silently
  readmit a future launder).
- **C. Lineage-aware alias-scope emitter.** A child's alias scope is nested within its
  ancestor's; never emit mutual-`noalias` between a child and any ancestor. Retire "distinct
  `&uniq` ⇒ mutually noalias." Latent today; becomes a live miscompile the instant any deferred
  form is admitted — fix before that.
- **D. `derived_from` precondition.** `derived_from` may be non-None only on an unbound
  call-scoped temporary, enforced at child creation, so a future result-transfer fails closed.
- **E. OWN-12 effect-row exemption (keep exact).** Exempt only the ancestor closure `{h}` from
  the effect-row overlap test; every non-ancestor live borrow is still checked.
- **F. Optimizer consumer discipline.** Derive the child's `noalias` from the child argument's
  per-call exclusivity, not by widening the parent's whole-function `noalias` scope.

## 5. Exact v0.6 → v0.7 delta (drafted; owner-gated — NOT applied here)

Mechanics: bump `spec/kernel-spec-v0.6.md` → `spec/kernel-spec-v0.7.md`, rename the file,
update the title and every live reference in one change (spec-guard hard-fails an in-place edit).

Spec rules changed:
- **OWN-5** — positive carve-in (child creation is the sole borrow admitted through a place
  overlapping `resolved(h)` while a child is live) + carve-out (h's read/write allowance
  suspended while a child is live), realized as the Condition-B SUSPENDED flag.
- **OWN-6** — statement-scoped child definition; eligible-holder restriction (parameter or
  let-bound borrow, no match binder); suspension and resume-at-statement-end.
- **OWN-12** — ancestor-closure-only exemption in the effect-row overlap check (Condition E).
- **PATTERNS P4** — "no reborrowing" → "bounded statement-scoped reborrow."
- **NEW numbered rule (Condition A)** — borrow-free / region-free storage.

Checker changes shipped with the delta (non-spec): unconditional exclusivity assertion (B),
`derived_from`-only-on-unbound-temporary (D), lineage-aware alias-scope emitter (C).

Deferred (unchanged; each needs its own reconciliation + review before admission): reference-result provenance
result-transfer, uniq→shared downgrade, uniq slices, affine-element buffers, match-binder
parents, grandchild chains, bound children.

## 6. Honest limitations and still-owed items

- **Bounded evidence, not proof.** The model-check is 1,000,000 random programs (failure-to-
  break), and the FR reconciliation is a structural argument from the calculus — neither is a
  closed-form proof. They are complementary.
- **Abstract model.** The model-check targets a focused model of the rule, not the production
  full-language checker. Folding the rule into `prototype/checker` and extending the shared
  oracle with exclusivity is production-integration work after the spec lands.
- **Two items owed during production integration (not pre-approval blockers):**
  1. **Backend emission audit** — run the real codegen and confirm what it keys `!noalias` /
     `!alias.scope` on for a reborrow argument. Can't be done yet: wfc does not lower reborrows
     (they are in the "unsupported" bucket). Conditions C and F pre-empt the risk; the audit
     confirms it when lowering is built.
  2. **Verbatim FR-paper pass for the `*w` reborrow edge** — the existing reconciliation
     covered the no-lineage core; the live parent→child edge wants a page-anchored pass
     classifying the lexical suspend as stricter-sound.
- **Transitive-escape closed by grammar, not by the model-check.** The model-check generator
  did not emit borrow-inside-an-`own`-result; Condition A closes that structurally (v0.6 can't
  express a borrow-typed field), which is why A is a NO-GO blocker.
- **Fragment only.** Soundness is established for the vacuous-ancestor fragment; every deferred
  form reopens the analysis and stays out.

## 7. What approval means / next steps

If approved: (1) draft the exact v0.7 delta (rules above + Condition A's numbered rule),
notify with the precise text, and land it as the gated change (version bump + rename + refs +
`make approve-spec`); (2) implement the fragment in the production checker with Conditions
B/C/D and add conformance cases; (3) grow wfc body semantics + lowering to classify the
reborrowing functions clean (the Phase-2 continuation), running the backend emission audit
then; (4) re-run both project gates and the widened model-check against the production checker.

If declined or deferred: wfc's reborrowing functions stay "legal but unsupported"; the
alternative is the Option-A analyzer-core rewrite (`DOSSIER.md` §3), whose cost was not
measured.
