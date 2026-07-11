# Review: requires-check-accounting-design.md

Reviewer: session of 2026-07-11 (three-lens panel: repo-reality fact-check,
governance red-team, constitutional complexity audit; synthesis by the main
session). Verdict and required changes below; the draft is NOT rejected — its
analysis is largely right — but four defaults must flip and the first slice
must shrink before anything becomes normative.

## Verdict

ACCEPT the analysis, REJECT the scope of the first slice. The draft's concept
separation (requirement / safety check / optimization guard / performance
acceptance), its decidability boundary, its ERR-4 classification of shortage,
and its writer guidance are correct and constitutionally grounded. Its factual
claims about current implementation state were verified against the code and
are accurate (only staleness: the corpus is 31/81 cases, not 28; the doc-only
requires rejection is missing from its FN-8 description). But the accounting
machinery as drafted is a governance system larger than the ~200-line prover
it governs, several of its defaults resolve fail-open where W3 demands
fail-closed, and one default actively creates the single incentive a
safety-first language must never create.

## Four blocking corrections

**B1. Approval identity must be per-site cone, not whole-artifact.**
Whole-artifact hashing invalidates every approval on every commit. In the D0a
workflow (one human approves an AI fleet's output) that guarantees
rubber-stamping — and a rubber-stamped `authorized-retained` record is worse
than no gate, because it launders unreviewed debt with false review
provenance. The draft defers cone-digests as a future optimization; that is
backwards. A stale approval cannot cause miscompilation (approval never
removes a check), so occasional cone over-approximation is the cheap failure
and guaranteed churn is the expensive one. First slice: bind approvals to the
site's dependency-cone digest + reason class, make reason classes
machine-checkable against compiler-confirmed context.

**B2. Retained-by-default-debt inverts to obligation-backed debt.**
"Every retained check is unaccounted debt" would block shipping measured-win
artifacts whose retained checks ARE the T1/T2 safety floor. The correct rule
is three-way, and it reconciles both review lenses:
- analyzer derived a discharging obligation, fact missing or mismatched ->
  HARD FINDING (fail-closed on known-dischargeable debt);
- analyzer affirmatively reports no obligation family applies -> PASS
  (intrinsic-dynamic; this is the safety floor, not debt);
- anything indeterminate — analyzer error, missing dependency record,
  backend-eliminated without verified provenance claiming credit ->
  UNACCOUNTED (fail-closed).
One principle covers it: at the performance gate, unresolved accounting
resolves to failure, never to credit — reject-when-unsure, applied to
accounting.

**B3. `explicit-unused` must never be a hard failure.**
A hard failure on unused explicit checks creates a direct incentive to DELETE
defensive checks to pass a performance gate — the one incentive this language
exists to make impossible. Invert: `explicit-retained` is counted and
cost-reported but PASSING; it escalates only with independent profile evidence
that it burns measurable hot-path cycles AND a proof/guard alternative was
available.

**B4. `versioned` is not automatically accounted, and drops out of slice 1.**
As drafted, guard versioning is the cheapest route to credit: any
recognizable shape gets versioned, the writer states no contract, ships double
code size, and passes. Fail-closed instead: versioned origins need an
independently measured hit rate above a pre-registered threshold AND a size
delta within the (not-yet-built) 13.3 budget, or an authorization record.
Since neither instrument exists, guard versioning stays experiment-only (as
section 14.5 itself says) and the `versioned` state leaves the first
accounting slice entirely. Additional reasons: input-dependent performance
cliffs cut against W1 predictability, and the repo's own scoped-alias data
(121 vs 2132 asm lines) is the code-size warning.

## The approved first slice (everything else deferred with named triggers)

KEEP (delivers the W1 value and the W3 property at a fraction of the
machinery):
1. Section 5 obligation-driven analyzer + section 11 diagnostics — the
   genuinely new, genuinely valuable contribution. Body -> obligation ->
   "first missing fact" / "first failed premise". The existing 31-case corpus
   becomes the diagnostic oracle.
2. Wire obligation findings into the EXISTING codegen-parity gate (it already
   pins per-site proved/retained counts and already runs in make check) —
   no parallel state cube.
3. Approvals as GATE-1 instances (answers review Q7: extends GATE-1, no new
   authority model), per-site cone identity per B1.
4. The section 9/10 writer doctrine graduates to PATTERNS.md now — "requires
   means invalid call, not uncommon call", the ERR-4 shortage table, and the
   overconstraint warning need no accounting machinery to be true.

DEFER (with trigger):
- `versioned` state + guard versioning -> trigger: 13.3 size budget built AND
  13.2 decoder experiment shows a versioned origin beating both plain
  retention and explicit requires on a real workload.
- Domain records + `domain-unauthorized` -> trigger: first data-dependent
  decoder domain, or the 13.4 authorship experiment landing the
  domain-shrinking attack. (The minimal slice already denies those attacks
  credit: an unrecognized or over-strong requirement simply earns nothing.)
- Lowering-state cube / backend-eliminated provenance -> trigger: first
  claim of backend-elimination credit at a promotion decision.
- Profile corpora / hit-rate infrastructure -> trigger: versioning
  reinstated. When built, split roles: pinned public corpus for audit
  reproducibility, held-out owner-rotated corpus for gate DECISIONS
  (a pinned decision corpus is Goodhart-able).
- Root-set closure machinery -> slice 1 scopes accounting to owner-selected
  promoted roots (answers Q1) with one tripwire now: a function that the
  independent profile shows hot but outside the accounted closure is a gate
  failure.

## Answers to the draft's review questions (short form)

1. Owner-selected root set; closure computed by the compiler, shown to the
   owner, never writer-proposed; hot-but-unaccounted tripwire.
2. No — versioned needs explicit permission plus measured hit rate and size
   budget; not in slice 1 at all.
3. Not automatic; outlining decisions wait for the 13.3 experiment.
4. Exact structural normalization only (D1a). A small affine/Presburger
   engine is the classic scope-creep door; it re-enters only as a separately
   verified checker with its own adversarial review.
5. "Necessary and sufficient" may be claimed only inside a closed fragment
   the analyzer fully models, and always scoped to the named site set — the
   draft's own wording is right; keep it.
6. Overconstrained requires: fail-or-approve, never silent credit (agree with
   draft).
7. Extends GATE-1. No second authority model.
8. Yes — strengthening a performance-gated requires is a reviewed domain
   change; while FN-8 is absent from fn_sig, the privileged statement of the
   accepted domain is the GATE-1 record itself (deferred with domain records).
9. Per-site cone digests from day one (B1); whole-artifact identity is
   rejected even as a first slice.
10. Callee-side check plus an entry in the boundary-debt register until
    boundary frames exist; no publication claim before then.
11. The validated-plan question (binding content-derived size to input)
    is real but out of slice; the prepare/required_output/decode_prepared
    shape is the right candidate and needs aggregate-result lowering first.
12. Keep `requires` provisionally (R3 register already marks it); the
    teaching-pack line carries the semantics; revisit at fn_sig integration.
13. QOI decode. Fits the D7 ladder, safe framing, fixed 64-entry table,
    bounded per-token burst (suits the 9.4 burst-guard family), no LZ window
    complications, real swap-in artifact at the end.
14. Pre-registered before the transform ships; decided with 13.3 data — not
    picked now.
15. Yes — but as the decoder experiment's deliverable, not a slice-1 blocker;
    slice 1's honest scope statement is "check debt only, allocation channel
    explicitly not covered yet" (B2 narrowing makes this honest).
16. Bounds only in slice 1. Overflow/allocation/explicit/transitive enter
    with evidence, each with its own adversarial pass.
17. Out of scope entirely; no reservation now (also per the safe-direction
    framing directive).
18. Backend-elimination credit requires verified provenance; until that
    exists the state is reported but never credited (subsumed by B2's
    fail-closed rule).
19. Only two roles in slice 1: boundary checks on gated entry points
    (mandatory) and everything else (explicit-retained, passing, per B3).

## Factual patches for the draft

- Corpus counts: 31 cases in output-capacity-lockstep, 81 total; n19-n25
  already cover part of the 13.1 matrix (guard-literal and requires-shift
  negatives shipped 2026-07-11).
- FN-8 description: note the doc-only-clause rejection (a requires reducing
  to zero effective statements is an FN-8 error).
- Section 13 additions: three governance-cost metrics — approval actions per
  artifact revision (target ~0 steady-state), gate precision (findings later
  judged legitimate vs false detentions), and time-from-finding-to-promotion.

## Where the draft is simply right (adopt verbatim)

The four-concept separation (section 4). The decidability section, including
"a missing requires is not a general language error". The guard-is-not-a-trap
rule. The differential audit mode (13.1) — the strongest honesty instrument
in the document. The threat-model row "encode the work as ordinary Result
control" honestly admitting incompleteness. The refusal to let optimizer
remarks substitute for artifact facts. And the recommended disposition's
restraint — which this review mostly just enforces against the draft's own
section 8.
