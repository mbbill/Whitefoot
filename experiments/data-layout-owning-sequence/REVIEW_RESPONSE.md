# Review response: E0.1 research report

Date: 2026-07-13. Scope: owner-advisor review of constitutional consistency,
cross-project design history, conformance classification, and decision framing.

Status: complete historical non-normative review disposition. D11 supersedes its
former owner-decision surface and authorizes no G0-Core work, family-lock work,
experiment, production implementation, specification change, scored timing, wfc
migration, PATTERNS change, default teaching, or external model disclosure. The
alternatives remain inputs to a future dense-family lock; this response selects
none and does not restart automatically.

## Verdict

**REVISE, WITH THE EMPIRICAL CORE RETAINED AND THE DESIGN RECOMMENDATION REOPENED.**

The report correctly separates layout, allocation shape, and initialization policy;
protects current SoA; rejects the unsound affine-fill prototype despite green tests;
requires isolated single-semantics toolchains rather than a feature flag; and refuses
to claim a performance result. Those conclusions remain valid.

The earlier recommendation of derived Flat storage plus a recursively fresh recipe
was underpriced. It conflated a previously carded declarative copy tier with automatic
structural Copy, did not engage the OWN-1 tag-only precedent, and did not account for
GRAM-9's prohibition on nested ordinary construction. The opaque sequence proposal
also understated that it reverses STOR-1. Those points reopen the language choice.

## Disposition of review findings

### R0 — The reviewed artifact and completed steps must be durable

**Finding: SUSTAINED. Exact source archived; gate discipline restored.**

The original review named a disposable worktree and retained only the patch hash.
That was insufficient: a hash can authenticate bytes but cannot recover them. Commit
`68a55e4` archives the exact 57,547-byte reviewed diff as
[`DETACHED_CANDIDATE.patch`](DETACHED_CANDIDATE.patch), records its base
`58baa71fb4c36a4728dd42aea6b05ce4be7aa0b1`, preserves SHA-256
`bed070414f9552ea105857404d6d1296b98542a28cc65fa6899a197830e6774e`, and adds the
matching decision-gates entry. The executable worktree remains deliberately
disposable; the reviewed source does not.

The revised report, this response, and their final gate entry are likewise committed
as one durable report-package step before the review disposition is treated as
complete.

### R1 — Declarative `copy struct` and automatic structural Copy were conflated

**Finding: SUSTAINED. Documentation corrected; design open.**

The 2026-07-10 project record carded a copy-struct tier in which an author may declare
an all-Copy-field struct Copy. That is an explicit, type-local opt-in. Automatic
structural Copy instead infers Copy for every structurally eligible record and changes
the canonical ownership spelling throughout existing source.

Both routes permit the existing Copy-only `buffer_new<T>` to initialize record
storage, but their visibility differs:

- automatic structural Copy has no declaration or call-site signal;
- declarative `copy struct` exposes the policy at the type declaration;
- both still make later bare uses implicit copies rather than call-site-explicit
  operations.

The report and candidate document now compare these separately. Neither is accepted
or rejected by this review.

The design tree remains unchanged while the owner choice is open. If a later
production decision selects affine Flat, automatic structural Copy, or any other
route that supersedes the recorded declarative-copy path, that same change must move
the old path into the appropriate `.alt/` branch and pair the reasons for selection
and supersession. If declarative `copy struct` is selected, its existing card must be
updated with the deciding evidence instead.

### R2 — The OWN-1 tag-only Copy precedent must be answered

**Finding: SUSTAINED. Safety and cost-visibility grounds separated.**

OWN-1 was amended so tag-only enums are Copy because resource-free affinity bought no
safety and forced an integer-state workaround with a measured 1.6-1.8x kernel loss.
A proposed Flat record is also resource-free. It is therefore insufficient to defend
affine records solely by saying that Copy is unsafe; for the narrow no-borrow/no-drop
class, bit duplication can be memory-safe.

Memory safety is not the only remaining distinction. A nominal all-Copy-leaf record
may encode one authorization state, protocol token, or private-constructor invariant.
Automatic structural Copy would duplicate that state even when doing so is
memory-safe, and would prevent future modules from relying on one-value authority.
Declarative Copy retains a type-author decision; affine Flat retains nominal
uniqueness by default. This is a correctness and maintenance axis.

Performance cost visibility is independent. A tag is small, while a record may be
24, 56, or many more bytes. Automatic Copy hides that cost everywhere; declarative
Copy exposes it only once; explicit Repeat/Clone can expose duplication at the
operation; affine Flat storage forbids it unless separately authorized. The
precedent frames the owner decision but does not decide it.

The revised documents link this issue to the copy-classification design record and
no longer present `Flat != Copy` as self-justifying doctrine.

### R3 — The recursive recipe conflicts with GRAM-9

**Finding: SUSTAINED. Recommendation withdrawn; five preliminary cost inventories
exposed.**

GRAM-9 requires every construct field value to be an atom. A nested record construct
must first be named by `let`. That named value is one affine record, so it cannot be
reused in a repeated outer fill. Consequently, a recursively fresh per-slot recipe
cannot be ordinary construction with no language delta.

The revised documents price five alternatives:

1. **Dedicated recursive recipe.** Preserves recursively nested affine Flat records,
   but adds a second recursive construction spelling, evaluation/effect rules,
   parser/AST/checker/diagnostic paths, and FORM-1/GRAM-9/META-5 debt.
2. **Single-level dedicated initializer.** Keeps Copy leaves as GRAM-9 atoms and has
   a smaller surface, but excludes nested record fields or forces source flattening.
3. **Per-slot builder or initialized-prefix owner.** Uses ordinary ANF construction
   and one move per slot, but imports partial initialization, failure atomicity,
   inaccessible-tail, borrow, and drop obligations from E0.1b.
4. **Explicit Repeat/Clone.** Makes duplication visible, but broadens semantics beyond
   storage and needs conformance, effects, fallibility/allocation, and cost rules.
5. **Declarative `copy struct`.** Reuses current `buffer_new<T>` and GRAM-9, but adds
   declaration grammar, all-Copy eligibility checking, TYPE-2 aggregate
   layout/lowering, and ownership-context conformance for assignment, calls, returns,
   and matches; every later use of the opted-in type is implicit Copy.

Automatic structural Copy has the fifth route's initialization convenience with an
even wider implicit ownership change. These entries are category-level lower bounds,
not completed META-5 derivations. No next prototype is selected. Any future
experiment must freeze one spelling, evaluation count, effect model, eligibility
boundary, and META-5 delta before implementation.

### R4 — Candidate findings were misclassified as design blockers

**Finding: SUSTAINED. Findings separated by provenance.**

- The affine-fill contraction is intrinsic to the rejected candidate and remains a
  blocker for that artifact.
- Index atoms skipping ownership flow was a pre-existing mainline OWN-1 bug. Commit
  `38d642e` fixed the first liveness/readability reproduction, not the complete
  ownership/type surface. Commit `7438e17` closes the offset, match, payload-borrow,
  try, and contextual-return seams with source-level
  conformance coverage. It is not an unresolved E0.1 design question.
- The match-scrutinee bypass was an independent mainline checker/conformance gap and
  is closed by `7438e17`. It provides no evidence for an
  E0.1 ownership route.
- Recursive projection tokenization was implementation drift from the intended
  projection surface. Commit `50a1ddd` repairs and pins it in stage 0 and wfc; the
  historical drift cannot justify excluding nested record storage or inventing a
  new design.

This classification preserves the value of hostile review: candidate review found a
real mainline defect, but the defect does not become evidence for the candidate's
language mechanism.

### R5 — The sequence proposal reverses STOR-1

**Finding: SUSTAINED. E0.1b reframed as a storage-taxonomy redecision.**

STOR-1 states that growable collections are future library structures over
`buffer<T>` plus ordinary aggregates and generics; they are not kernel constructs.
An opaque kernel `sequence<T>` therefore does not merely complete an unfinished
implementation. It reverses a recorded design direction.

Two branches must remain visible:

- preserve STOR-1 and determine the smallest checked storage operations required for
  a library owner to maintain an inaccessible uninitialized tail; or
- reopen STOR-1 and consider a kernel sequence.

The second branch owes a full META-5 declaration: tokens, rules, spellings,
exceptions, storage class, ownership, borrowing, growth, allocation failure,
destruction, diagnostics, teaching, and evidence-selected ground. If approved, it
also requires a same-change design-tree redecision. This review does not approve it.

The original serial E0.1a-before-E0.1b protocol remains valid only for routes that do
not require a builder. Selecting a builder deliberately couples the phases and
requires a newly scoped protocol. The revised report asks the owner to decide this
rather than treating serialization as mechanism-independent.

### R6 — PATTERNS guidance is a production and default-teaching gate

**Finding: SUSTAINED. Gate made explicit.**

PATTERNS P2 currently blesses the append-only, index-linked SoA pool and names its
cache/vectorization and scoped-alias advantages. A technically sound AoS capability
cannot silently replace or ambiguate that closed taught catalog.

Before any record-storage route enters production, owner-reviewed pattern guidance
must state whether AoS is blessed, the measured row/column/mixed access conditions
that select it, and the canonical fallback. The catalog must remain COMPLETE and
EFFICIENT. Default teaching additionally requires the separately authorized,
benchmark-blind low-tier writer panel. An expert AoS win is insufficient for either
gate.

No PATTERNS edit is made by this review because no design has been selected.

### R7 — Float exclusion was overread

**Finding: SUSTAINED. Limitation narrowed to the disposable prototype.**

The prototype excluded float fields because its stage-0 layout path was deliberately
narrow and hand-maintained, not because the language evidence showed `f32` or `f64`
to be non-Flat. Existing primitive float buffers remained byte-identical. The revised
documents state that float eligibility is unresolved production scope and must be
derived from target DataLayout if a Flat route is selected.

Other prototype exclusions likewise remain experiment limits unless separately
justified; they are not silently promoted into language law.

### R8 — Empirical and process conclusions remain valid

**Finding: AFFIRMED.**

The following evidence survives the revisions:

- Whitefoot currently cannot express fixed `buffer<Record>` AoS storage;
- wfc's current SoA has plausible workload and memory advantages and remains the
  protected baseline;
- natural fixed AoS would increase Token/Ast requested bytes by 11.1% and total
  frontend requested bytes by 3.85% under the same capacity policy;
- the 64-bit row/field/scalar lowering shape is feasible without changing frozen SoA
  raw LLVM;
- no scored performance, capability-adoption, wfc-migration, or default-writer claim
  exists;
- layout, allocation shape, initialization policy, capacity trajectory, and owner
  lifetime must remain separately attributed;
- a feature flag is not an admissible experiment or production mechanism.

The static utilization numbers also justify continued interest in initialized-prefix
storage: the baseline requests 204.13 MiB for the canonical 1 MiB source while the
Token/Ast live prefixes use about 10%-21% of capacity. They do not by themselves
select a kernel sequence or establish a runtime win.

## Disposition of the original owner questions

1. The evidence supports treating fixed record storage as a real capability gap.
2. The prior request to approve no-marker Flat storage is reopened; declarative Copy
   and automatic structural Copy must be distinguished first.
3. The recursive-recipe recommendation is withdrawn pending the GRAM-9 and META-5
   comparison above.
4. Prototype surface exclusions, mainline conformance fixes, and intended language
   restrictions are now separate questions.
5. Strict E0.1a/E0.1b serialization depends on the selected initializer; a builder
   route would require deliberate coupling.
6. Scored timing remains blocked on endpoint, lifetime, attribution, correctness,
   and hostile-review gates.
7. Capability adoption, wfc migration, pattern doctrine, and default teaching remain
   independent approvals.

At the time of this review, items 2-5 required owner selection. D11 supersedes
that direct selection surface. This review records no preferred language route.

## Superseded next state

The state recorded by this historical response was reviewed research plus
separately authorized conformance repair:

- retain the detached prototype solely as rejected feasibility evidence;
- do not implement another language candidate until the owner selects and freezes
  one alternative;
- retain the independently authorized checker and parser repairs as enforcement of
  current rules, not as E0.1 candidate semantics;
- do not start scored timing;
- do not open a production E0.1/STOR-1, PATTERNS, specification, stage-0, wfc, or
  teaching change from this response.

The conformance repairs remain complete current-language work. D11 supersedes
the direct owner-selection next step: the owner will separately discuss whether
to authorize bounded G0-Core work, and any later dense-family Lock A must
explicitly retain, revise, or supersede the relevant alternatives and measures
here. This response authorizes no further work.
