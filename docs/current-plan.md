# Current Plan

Status: ACTIVE (owner selection 2026-08-09): complete the selected
obligation-discharge direction before returning to wfgrep. This replaces the
mixed attribution/spec-batch plan selected on 2026-08-06. The credited wfgrep
checkpoint remains intact but PARKED until the direction reaches its completion
boundary.

This ACTIVE plan authorizes only the current independently reviewable step
below. The owner selected the later capability sequence and its stop condition,
but each later slice must still replace this rolling plan before execution.
Task 0046 closed the bounded provenance review positively after task 0041's
negative measurement. The revised rule remains held behind stage 7 and O3;
this replacement plan resumes the owner-selected sequence with the smallest
counted range loop earned by the SHA-256 and writer evidence.

Derived from: [Direction Outline revision 25](roadmap.md), items `PROOF-8`
(primary), `BOUND-1`, `VERIFY-1`, and `VERIFY-2`; `CAND-8` remains the selected
flagship but is parked by this owner selection.

## Direction and current milestone

Items 1–4 of the dossier's §8 sequence are shipped: the claim construct, the
normative L0 entailment fragment, caller-side OP-4 index discharge, and the SYS
count postconditions with ENT-3 S10 fact introduction. v0.24 corrects ENT-5,
uses the stable active-spec filename, and has an installed frozen acceptance
baseline. Stage 5a-R now has a positive held design: all three canonical
provenance sites are classified, the finite call relation retains its concrete
leaf, and the material `requires` bypass still prevents activation. The current
milestone is stage 6: add one structural counted `u64` range whose bounds enter
L0 without general loop induction.

The selected direction after this measurement is a counted range loop,
`requires` as a call-site goal, provenance gate closure, `ensures`
with the fact sources its real examples need, and finally claim-ledger tooling
plus an opt-in `deny-claims` partition. Those entries must reach the normal
specification, compiler, conformance, and real-program paths or return a
reproduced blocker for owner disposition. This selected boundary does not
claim that `claim` becomes the sole trap source for the whole language: bare
trapping arithmetic and ordinary explicit checks remain outside it.

Further wfgrep attribution or implementation is not part of this milestone.
It resumes only after the complete selected direction is active and verified,
or after the owner disposes a reproduced blocker that stops the sequence.

## Current step — stage 6 counted `u64` range loop

### Why

The installed SHA-256 program has nine index obligations and proves none
without four loop claims. All nine occur in unit-stride ascending walks whose
body needs only the structural statement that its loop value lies in one
half-open range. Three of four hostile writer probes independently chose the
same `for i in a..b` shape. A counted range can therefore remove the dominant
claim family without adding general induction, a widening interval engine, or
another arithmetic term language.

### Do

1. Register task 0047 from the terminal task-0046 closure before substantive
   work. Consult the live proof and surface-form design nodes and their real
   alternatives; ordinary `loop @l` remains available and unchanged.
2. Add exactly one counted form, canonically
   `for @label i in lower..upper { ... }`. Both endpoints are `own u64` atoms,
   each restricted to an [ENT-2] term or constant and evaluated once from left
   to right into compiler-owned immutable captures. A subscripted or otherwise
   non-term endpoint must first be rebound through an ordinary checked `let`.
   Add `..` as one fixed compound terminal with attached canonical rendering.
   A digit-started numeric candidate stops before `..`; one dot retains its
   existing float/member behavior. Thus `0_u64..1_u64` partitions as literal,
   range terminal, literal rather than one NumberForm, with hostile seam tests
   fixing that maximal-munch boundary. `for` and `in` become fixed lowercase
   terminals and therefore leave [FORM-3] identifier eligibility. Inventory
   declaration and use positions across the protected corpus, real programs,
   and live experiments, record the exact accepted-program narrowing, and do
   not silently migrate a collision; the expected zero-site result is a
   measured acceptance condition rather than an assumption.
   The form is ascending, unit-stride, and half-open; `lower >= upper` executes
   zero iterations. There is no descending range, step, iterator protocol, or
   `continue` in this slice.
3. Make `i` a compiler-updated immutable `own u64` binding visible only in the
   body. It is not a writable root: source cannot `set` it, form a unique borrow
   of it, or pass any projection of it to a callee write. Shared reads and
   borrows retain their ordinary rules. Because the body may execute more than
   once, it inherits both [OWN-11] repeated-body restrictions: it cannot move an
   affine binding declared outside the counted loop, and any borrow expression
   inside it must name a region declared inside that body. This extends the
   existing judgment to the new production without weakening ordinary loops.
   The mandatory label participates in the existing label domain and
   `break @label` rules. A normal body fallthrough advances once; a matching or
   enclosing `break`, `return`, or `propagate` error edge does not. Body-local
   cleanup and nested labelled exits use the existing edge-carried release
   path. On normal fallthrough, body-scope
   teardown and reverse-order cleanup finish before the hidden increment and
   backedge. A matching or enclosing break, return, and propagated error each
   perform their owning cleanup exactly once, while guard-false exhaustion
   performs no second body cleanup. In particular, a body-local shared borrow
   of `i` ends before the next hidden update. The hidden increment is defined
   only after `i < upper`, so `upper = u64::MAX` never wraps or adds a
   writer-visible trap.
4. Extend L0 only with finite compiler-owned endpoint-capture terms and the
   counted-loop structural source: capture equalities at the preheader, then
   `lower_capture <= i` and `i < upper_capture` at every body entry. Captures
   have identity derived from the counted-loop node and endpoint side. Existing
   closure, support, kills, and S7 constant-offset rules do the rest; ordinary
   loops gain no induction and no fact is attached to a mutable endpoint's
   later value. Define the counted continuation separately from `loop_stmt`:
   join the zero-trip/guard-false exhaustion edge with every `break` naming this
   counted label, after edge cleanup and scope-exit kills. Breaks to enclosing
   loops and function exits do not join there. The iterator and capture terms
   leave scope before the join, so no body structural fact or `i = upper`
   postcondition escapes, and a break-free counted loop has a real exhaustion
   edge rather than ENT-5's contradictory empty join. The checked/lowered
   header carries outer state plus captures and `i`; the continuation and every
   local break carry outer state only, so the existing ordinary-loop carried
   list cannot leak the new binding or captures.
5. Follow the stable-spec workflow for the smallest complete v0.25 delta and
   outgoing immutable v0.24 archive. Verify grammar through both runtime paths,
   independently review the complete stable-file diff and digest under the
   owner's 2026-08-09 delegated branch-revision authority, then implement the
   parser, checked representation, resolution, semantic facts, ownership and
   effect checks, typed lowering, backend path, diagnostics, and generated data
   through one general construct. Do not add a governance candidate copy.
6. Migrate only the three counted index loops in `sha256_abc.wf`; retain its
   unrelated ordinary loop. Add project-independent positive, negative,
   near-miss, cleanup, and conformance cases. Update active identity, derivation,
   writer documentation, outline, plan, and design memory only when the exact
   candidate is activated. The protected
   `gram6-pos-no-operators` source doc plus manifest reason/doc are already
   stale about infix arithmetic and `if`, and would also become false about
   `for`; rederive those three prose fields from v0.25 as one explicitly
   reviewed existing-corpus change while keeping the stable id, GRAM-6 rule,
   and run verdict unchanged. No other protected source or verdict may change
   silently.

### Verify and accept

- Native grammar verification and canonical round-trip cover the exact new
  tokens, `for`/`in` fixed-word reservation, numeric/range seams, mandatory
  label and binding positions, endpoint atoms, nesting, and rejection of every
  noncanonical or out-of-scope spelling. A lexical-role census records every
  former identifier declaration/use collision in the protected corpus, real
  programs, and live experiments; zero is accepted only when the census is
  reproducible, and any nonzero unplanned narrowing stops review.
- Runtime controls cover empty and reversed ranges, `0_u64..1_u64`,
  `18446744073709551614_u64..18446744073709551615_u64`, and
  `18446744073709551615_u64..18446744073709551615_u64`; endpoint bindings
  mutated inside the body do not change the captured trip count. Nested
  current/enclosing breaks, return, propagated error, and body-local affine
  cleanup follow their existing edges. Hostile [OWN-11] controls reject an
  outer affine move and a borrow naming an outer region while accepting the
  corresponding body-local forms. A set targeting `i`, a unique borrow of `i`,
  a callee write through it, a non-`u64` or non-term endpoint, an endpoint use
  of `i`, an unknown/duplicate label, shadowing, and a post-loop use of `i`
  reject at their owning existing or new rule.
- Semantic controls distinguish the counted form from an ordinary loop. At
  every counted body entry they derive both structural bounds, retain the
  safe `u64::MAX` increment argument, compose through S7 constant subtraction,
  and kill facts only when their real support is invalidated. Mutating an
  endpoint source binding cannot retarget a capture or manufacture a fact.
  A carried `j`, an access at `i +wrap 1` against the same upper bound, an
  upper endpoint wider than storage without an independent
  `upper_capture <= len(storage)` fact, and an `i -wrap k` whose lower endpoint
  is too small remain unproved. A counted loop
  with no break leaves a reachable non-contradictory continuation; a zero-trip
  path imports no body fact, and an early break creates no exhaustion fact.
  In particular, an empty or reversed break-free range followed by an otherwise
  unproved out-of-bounds access still rejects rather than inheriting ordinary
  loop's empty-join contradiction.
- The migrated SHA-256 function proves all 9/9 subscript obligations without
  S2/S3, removes exactly four claims, changes its exhibited effect from
  `traps` to `pure`, emits no `wf_trap`, and directly validates
  `sha256_abc_word_zero() == 3128432319_u32` (`0xba7816bf`) rather than relying
  only on the existing 1024-iteration wrapping aggregate. Its sustained outer
  loop remains the ordinary-loop control; the existing rotate and
  schedule-address code-shape checks remain. Rerun the installed acceptance
  buckets and report the exact residual induction demand rather than
  extrapolating.
- Lowered checked IR and native execution contain no counted-loop overflow or
  bounds fallback for the proved sites. All unrelated real programs and
  existing conformance rows retain their verdict, cited rule, and observable
  behavior; additive cases exercise the new form through the normal adapter.
  The ordinary-loop-only wide-probe recognizer is unchanged and receives no
  counted-range credit in this stage.
- The archive-integrity gate, active-spec chain, two grammar paths, focused
  frontend/semantic/lowering/backend tests, complete compiler check, complete
  repository gate, ignored adapter tally, and MCTS lint all pass on the final
  tree.

### Accept and stop

This plan is terminal when the exact counted-range specification and one normal
compiler path are active, the SHA-256 9/9 result and hostile boundaries are
recorded, and no other verdict drift remains. If endpoint snapshots cannot be
given finite checked identity, the `u64::MAX` edge needs a hidden runtime trap,
or the nine real obligations require general induction, record the smallest
reproducer as a blocker rather than weakening the range. A positive closure
advances to stage 7; it does not activate the held provenance gate.

## Owner-selected roll-forward — dependency map, not execution authority

The following map preserves the owner's final objective and prevents later
work from silently changing order. It does not authorize any later `Do`.
Before each slice begins, reread its evidence, make it the sole current step in
a replacement plan, and preserve the normal candidate, impact, exact-approval,
activation, and real-program loop. A negative prerequisite measurement returns
for owner disposition; it is not permission to weaken a feature or skip ahead.

### 7 — `requires` as one atomic call-site goal

Replace the unconditional concrete callee-entry check with call-site
obligation discharge while preserving failure behavior at every implemented
entry boundary. The first version is explicitly one atomic predicate: FN-8's
single final check cannot thread multiple relations, and O11 boolean
composition is not selected. A second condition such as the remaining
`store_dynamic_length` premise therefore stays unsupported unless a later
owner-selected composition rule supplies it. The slice must close provenance
O3 before the gate can activate. A direct foreign-entry test is required only
when a real callable foreign boundary exists; a stub is not evidence for a
synthesized adapter.

### 5b — provenance gate activation

Task 0046 supplies the positive held rule review, but only after stage 7 closes
the `requires` bypass may a replacement plan propose provenance propagation and
the signature column. A constrained subject classified external by that
explicit-dataflow policy then must be handled by a value branch rather than
hidden in an aborting claim. Calls and adapters must preserve the policy through
one ordinary semantic path.

### 8a — postcondition proof-feasibility prerequisites

Freeze the smallest additional fact sources required by the two real examples
before promising `ensures`. `read_bits` needs a verified mask/bitwise bound and
an outcome-sensitive normal-result form; neither follows from current L0.
`append_slice` needs a fact connecting its loop-carried result to capacity, and
counted-range structural bounds alone may be insufficient. Measure first. If a
small structural rule cannot establish these obligations without the excluded
general induction or arithmetic entailment, return that reproduced blocker
instead of hiding a new proof engine inside `ensures`.

### 8b — `ensures`

Once 8a has an authorized fact fragment, add the smallest postcondition
language that exposes only verified normal-return facts to callers. Exercise
branches, early exit, cleanup, generics, unsupported forms, and false
postconditions. Acceptance requires the real `read_bits` and `append_slice`
obligations to discharge through the normal compiler path.

### 9a — deterministic claim ledger

Generate a deterministic read-only report from checked-program state for every
remaining named claim: obligation, provenance, justification, and stable source
identity. Clean builds must reproduce order and counts. This tooling lands
before any language marker.

### 9b — opt-in `deny-claims` partition

Design and then implement the language marker using the ledger evidence. Its
meaning must be transitive across ordinary calls and generated adapters, and
must specify how direct `claim`, ordinary trapping `check`, and callees that can
claim are treated; otherwise the partition is trivially bypassable. Ordinary
code retains the existing claim lifecycle, while the strict partition requires
each covered obligation to prove or take a value branch. This is not global law
and does not eliminate true-but-unprovable residue from the no-search checker.

## Stable specification rule

The ENT-5 activation is the one-time switchover. v0.23 remains immutable at
`spec/kernel-spec-v0.23.md`; v0.24 becomes the active
`spec/kernel-spec.md`, with no `spec/kernel-spec-v0.24.md` beside it. Every
later candidate edits the stable file on its task branch and is reviewed as a
diff plus complete digest. Its approved atomic activation first creates the
outgoing flat archive `spec/kernel-spec-vN.md`, failing if that path exists,
and installs the new approved bytes at the unchanged stable path. The active
specification is never renamed to follow its version; archived versioned files
remain absolutely immutable.

## Cross-stage invariants

- One normal semantic and lowering path; no program-, corpus-, function-, or
  test-shaped behavior.
- A fact may widen discharge only when the normative entailment rules derive
  it. Required checks remain unless proof discharges their exact obligation.
- Expected or externally caused failure is a value path; a claim is reserved
  for a broken program invariant and remains an executed runtime check.
- Protected conformance expectations never change without explicit owner
  approval. Unsupported compiler capability never becomes source rejection.
- Each activated slice restores the complete gate and reruns the real consumer
  that earned it before the next slice begins.
- Durable decisions and rejected alternatives stay synchronized through the
  `mcts-mem-use` workflow; task records carry progress, not authority.

## Direction completion boundary

Wfgrep remains parked until the selected §8 entry points above are implemented
end to end, covered by positive, negative, near-miss, and invalidation evidence,
exercised by their named real programs, and recorded in the outline; the
complete repository gate is green; and remaining claims and unsupported gaps
are reported honestly. A reproduced prerequisite blocker returned for owner
disposition is the only earlier stop. This boundary is narrower than making
`claim` the language's universal sole trap source.

## Explicitly outside the current step and selected boundary

O11 boolean-composition precision, general loop induction, arithmetic-term
entailment or arithmetic-mode dissolution, struct/witness invariants, the
OWN-3 predicate widening, and move-on-copy generic policy are not hidden parts
of this sequence. Their recorded triggers and approval requirements remain.
No wfgrep profiling, optimization, traversal, parallelism, or new system-family
run occurs while this direction is active.
