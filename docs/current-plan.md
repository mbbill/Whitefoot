# Current Plan

Status: ACTIVE (owner selection 2026-08-09): complete the selected
obligation-discharge direction before returning to wfgrep. The credited wfgrep
checkpoint remains intact but PARKED until the direction reaches its completion
boundary.

This rolling plan authorizes only stage 7 below. Task 0047 closed the counted
range milestone positively: exact v0.25 is active, the installed frozen
acceptance reproduces the reviewed result, and the complete repository gate is
green. The later provenance, postcondition, ledger, and strict-partition steps
remain a dependency map rather than execution authority. Before any future
specification approval request, the lead must first give the owner a complete
plain-language explanation, stop, and wait for an explicit response.

Derived from: [Direction Outline revision 27](roadmap.md), items `PROOF-8`
(primary), `BOUND-1`, `VERIFY-1`, and `VERIFY-2`; `CAND-8` remains the selected
flagship but is parked by this owner selection.

## Direction and current milestone

The shipped path now includes the named claim construct, normative L0
entailment, caller-side OP-4 discharge, SYS count facts, the corrected ENT-5
continuing-kill rule, and the v0.25 counted `u64` range. The installed SHA-256
worker proves all 9/9 schedule accesses without a claim or hidden trap. The
held provenance design reaches all three canonical Huffman subjects but cannot
activate while a callee `requires` block can hide the same protected obligation
behind an unconditional entry trap.

The current milestone is stage 7: make each admitted FN-8 block one atomic
call-site proof goal. Ordinary source calls must prove it before transfer and
callee effects; the callee body receives the proved predicate and no longer
executes an unconditional prologue check. Real process entry keeps a checked
boundary. This closes provenance bypass O3 but does not activate provenance.

Further wfgrep work remains outside this milestone. It resumes only after the
complete selected direction is active and verified, or after the owner
disposes a reproduced blocker that stops the sequence.

## Current step — stage 7 `requires` as one atomic call-site goal

### Why

The current executable callee prologue admits every ordinary caller and traps
inside the callee. Moving a protected subscript into a helper with a matching
`requires` therefore preserves the trap while bypassing a caller-side
provenance policy. Conversely, simply restricting FN-8 to one L0 integer
relation would delete existing legal declaration forms: the protected equality
case and the real base64 capacity predicate are pure, total single predicates
but not both representable as current ENT-2 terms. The smallest compatible
boundary is one finite typed predicate identity, treated atomically, with an
optional projection only when that same root already is one L0 relation.

### Do

1. After this plan and task-0047 closure land, register task 0048 in a separate
   lifecycle commit before substantive work. Consult the live
   `requires-entry-contract` and `obligation-discharge` design nodes plus their
   real rejected alternatives. The current callee-entry mechanism is a genuine
   predecessor: if stage 7 activates, move it into design history with the
   required paired re-decision record rather than silently overwriting it. At
   that same activation boundary, reconcile the live root, proof-doctrine, and
   effect nodes whose present Items still say that the executable entry check
   is never removed or contributes the callee's `traps` effect. In particular,
   update `whitefoot`, `checks-and-proofs`, and `effects` alongside the two
   named design nodes, while leaving recognizer-driven elision frozen as a
   rejected alternative. Do not edit design memory merely because this plan
   selected the future work; it changes only when the language and compiler
   change actually become live.

2. Preserve the complete current FN-8 declaration surface: zero or more
   clause-local lets followed by exactly one pure, total, non-trapping Bool
   check. Alpha-expand those lets into one finite typed `GoalTemplate`, then
   instantiate its formal datums at each concrete use. Predicate equality is
   only the resulting typed expression: selected operation rows, type and const
   arguments actually present in that expression, written operand order,
   formal-parameter ordinals with field/deref projections before call
   substitution, named-const declaration identity, and typed literals. A
   callee-instance id, final-check NodePath, and local spelling are diagnostic
   or provenance identity, not predicate equality. Two concrete instances may
   therefore share evidence only when substitution leaves exactly the same
   typed predicate; an instance whose substitution changes the expression does
   not match. Do not commute operands, fold
   constants, reassociate, invert comparisons, apply De Morgan, or eliminate
   double negation. A complete `band`, `bor`, or `bnot` DAG is one indivisible
   goal; its children are not facts. When the root itself is exactly one
   existing ENT-3 integer relation over substituted ENT-2 terms/constants, the
   ordinary L0 closure may prove that one goal without creating a second goal
   language.

3. At an ordinary source call, finish callee resolution, concrete generic
   instantiation, named-argument and type checks, borrow feasibility, and every
   obligation belonging to an actual expression first. Then substitute formal
   parameter datums with the caller's pre-transfer actual images, including
   resolved referents for borrow formals. Discharge the resulting single goal
   in the state entering the call, before any argument consume/borrow commit
   and before the callee's write or other effect kills. A successful call then
   follows the existing transfer and normal-return order. A failed or refuted
   goal is a compile-time call-site rejection; never insert an ordinary-caller
   fallback check.

4. Add a finite signed opaque-goal fact class alongside L0. A true/false branch
   establishes the corresponding sign; a passed explicit check or claim
   establishes the exact complete goal as true; and a callee body entry receives
   its verified requirement as true. Keep both a condition binding's own truth
   and its valid unique pure/total origin expansion, so a later mutation of an
   origin does not erase the already-computed Bool value while it does kill a
   reread predicate. Support is the union of resolved places read by the goal;
   `len(P)` retains ENT-5's length support boundary. Existing overlap writes,
   projected callee writes, consumes, holder/region exits, and lexical exits
   kill supported opaque facts. Joins retain only identical signed facts common
   to every incoming state, and ordinary/counted continuing-kill behavior is
   reused without induction. The combined L0/opaque state is contradictory when
   L0 is contradictory or one goal has both signs. At such an unreachable point
   every L0 relation and every signed goal is derivable, a call goal is
   discharged rather than refuted, and an all-derivable input does not constrain
   a nonempty join; an empty join remains all-derivable. At a non-contradictory
   call point, true present means discharged, true absent plus false present
   means refuted, and neither sign means unproved. No opaque fact decomposes
   into L0 subrelations or composes L0 relations into a Boolean DAG; O11 remains
   outside this plan.

5. Replace the executable ordinary-function prologue with an admitted-body
   axiom. S4 supplies the complete atomic goal at body entry and also supplies
   its one L0 relation when the root has that exact projection. Later body
   writes and consumes kill it normally. Direct recursion, mutual recursion,
   forward calls, and concrete generic instances use the same finite inventory:
   every call edge must prove its own instantiated goal, so no recursive fixed
   point or caller-order exception is introduced.

6. Redefine effects consistently. A `requires` declaration is a signature
   obligation, not an executed body occurrence, and contributes no `traps` or
   memory effect to the callee row. A pure body with a requirement may therefore
   remain `pure`. An explicit caller `check` or `claim` still exhibits `traps`
   in that caller, and a trapping body remains trapping for its own reasons.
   Neither a proved goal nor S4 becomes `llvm.assume` or another optimizer fact.

7. Preserve failure behavior at every implemented entry boundary rather than
   inventing a foreign path. The compiler currently has exactly two real
   process wrappers: unlabelled `main()` and command `main(argc, argv)`, each
   calling one internal Whitefoot body. Lower the typed pure goal directly in
   the wrapper after ordinary argument/input setup and before the body; do not
   materialize an ordinary or IR helper function that accepts the source
   owners. The wrapper remains the sole owner of every `Args`, `DirectoryRead`,
   and `Output` value while this private evaluation performs only the same
   non-consuming reads as the admitted FN-8 expression, owns no source value,
   and carries no drop or release. On success the wrapper transfers each owner
   exactly once to one body invocation. On false it emits the original OP-5
   trap, invokes the body zero times, and follows EFF-4 rather than a second
   cleanup path. Source calls to unlabelled `main` use ordinary static discharge
   and never the process wrapper check. Keep every Whitefoot function internal
   and keep one external `@main`.

   The language's gated-foreign boundary promise remains: if that currently
   unsupported callable path is implemented later, its compiler-owned adapter
   must evaluate the same complete goal before the body. Stage 7 adds no FFI,
   export, or foreign stub and may not present one as boundary evidence.

8. Close provenance bypass O3 structurally without activating the held gate.
   Preserve the held protected-leaf identity `(concrete callee instance, exact
   ENT-6 obligation occurrence, normalized conjunct ordinal)` and add the one
   requirement identity `(same concrete instance, final-check NodePath,
   conjunct 0)`. Derive a finite bridge relation from requirement occurrences
   to protected leaves together with held PRV-2, using exactly two monotone
   generators:

   - For each local protected body leaf, compare its unasserted body state with
     S4 present against the same state with both the atomic S4 goal and its exact
     L0 projection omitted. If only the former discharges the leaf, add the
     local requirement-to-leaf bridge. A leaf proven without S4 needs no bridge;
     a leaf unproved even with S4 remains an ordinary ENT-6 rejection.
   - For each ordinary call whose callee requirement already bridges to an
     inherited protected leaf, perform the same S4-present/S4-blinded comparison
     for that instantiated call goal in the caller. If only the caller's S4
     proves it, add a bridge from the caller's requirement occurrence to the
     inherited leaf and retain the call plus downstream requirement occurrence
     as its witness predecessor. If the call goal is proved without caller S4,
     the bridge chain ends at that real evidence; if it needs S2/S3 instead, the
     later gate owns a local violation at this call rather than manufacturing a
     bridge.

   Solve these generators and ordinary parameter-datum call composition to a
   least fixed point over the finite concrete instances, requirement
   occurrences, calls, and protected leaves. A recursive or mutually recursive
   component with no local protected-leaf seed stays empty, and witness paths
   are reconstructed after convergence rather than stored in the lattice.
   This makes a two-hop or longer wrapper expose its own callable goal to its
   caller without requiring that caller to reconstruct a downstream helper's
   clause locals.

   Derive parameter datums exclusively from each protected obligation's
   constrained-subject dependency and compose those datums through calls; never
   use every place mentioned by a goal. A bound or base operand therefore does
   not become a subject merely by appearing in `requires`. At a call, retain the
   current callee requirement occurrence, inherited protected leaf, and composed
   subject datum. One argument produces one later PRV-2 event even when several
   datum/leaf pairs explain it. Stage 7 call acceptance uses the full caller
   state. The subsequent provenance gate will require that current instantiated
   atomic goal from the caller's unasserted state, excluding S2/S3, whenever the
   bridged protected subject is external. Thus a caller claim/check cannot regain
   the helper bypass, while a real dominating value branch or L0 allocation
   equality may prove the goal. This task must retain the converged bridge
   metadata and frozen rewalk needed for that next decision, but it must not
   reject provenance-tainted claims yet.

9. Inventory every active FN-8 declaration, call, effect row, backend prologue
   assertion, and protected case before fixing exact v0.26 bytes. The selected
   dispositions are:

   - preserve `fn8-pos-requires-eeq` as a runnable equality requirement and add
     exact caller evidence rather than narrowing FN-8;
   - preserve `x-base64-rfc-vectors-run` and its full capacity DAG, adding
     explicit exact evidence before each call rather than a recognizer or a
     weakened contract;
   - repurpose `fn8-neg-requires-missing-traps` so its EFF-2 subject is a
     caller-executed check/claim whose row omits `traps`;
   - move `fn8-trap-requires-false` to the real process-entry requirement path,
     preserving its OP-5 runtime-trap subject rather than fabricating a foreign
     caller;
   - change requires-only helper rows to `pure` where the body exhibits no
     other effect, including the affected positive FN-8/S4 cases; keep
     `fn3-neg-requires-member` focused on FN-3 by removing the stale incidental
     prologue effect; and
   - update `x-requires-output-capacity-run` and backend prose/assertions from
     “callee prologue” to caller discharge plus the body-entry axiom.

   Any additional protected source, verdict, cited rule, status, or observable
   behavior change stops the task for explicit review. All named protected
   changes enter the v0.26 owner packet; none is silently rewritten.

10. Draft the smallest complete v0.26 delta at the stable
    `spec/kernel-spec.md` path, prepare the exact outgoing v0.25 archive, update
    compiler, conformance, generated data, derivation, writer documentation,
    active pins, roadmap, plan, and design memory, and exercise the normal
    frontend-to-backend path. The branch candidate remains non-authoritative.
    Before activation, present the owner with a Chinese explanation of the
    exact language behavior, implementation, real-program result,
    accepted-set/protected impact, archive action, limitations, and complete
    digest; then stop and wait. Only explicit approval may append the chain and
    atomically activate. Never create `kernel-spec-v0.26.md` while v0.26 is
    active.

### Verify and accept

- Goal identity controls accept alpha-renamed and differently shared local-let
  DAGs but reject operand swaps, reassociation, comparison inversion, De Morgan,
  double-negation, different named-const identities, different operation rows,
  and any concrete generic substitution that changes the typed predicate.
  Distinct instances whose instantiated predicate bytes and types are identical
  deliberately share that predicate fact while retaining distinct diagnostic
  and provenance occurrence identities. Proving both children of `band` does
  not prove the `band`, and proving the `band` proves neither child.
- Exact dominating branch, check, and claim evidence discharge the same whole
  goal. False evidence refutes it; non-dominating or one-arm evidence does not
  survive a join. Existing L0 may prove an exact single relation. Writes,
  projected calls, consumes, and holder/scope exits kill only their real
  support; element writes do not kill an unrelated length fact.
- Call ordering controls cover moved actuals, borrowed referents, actual
  subscript obligations, and a first accepted call followed by a killing write
  and rejected second call. Forward, recursive, mutually recursive, and generic
  calls are traversal-order independent.
- Ordinary callee checked IR and LLVM contain no requirement check, entry
  branch, fallback trap, `llvm.assume`, or body clone. S4 still discharges body
  obligations. Exact effect rows accept a pure required body and reject both an
  omitted caller check/claim trap and a padded callee trap row.
- Both real process entry forms execute the goal once after their existing
  setup and before the body. A false goal produces the original OP-5 record and
  runs the body zero times; a true goal runs one goal and one body, with releases
  exactly once. Command-entry canaries cover true and false goals with owned
  `Args`, `DirectoryRead`, `command.stdout`, and `command.stderr`, proving no
  duplicate consume, drop, release, or body call. Startup failure still precedes
  the goal. The module retains one external `@main`; no fake foreign-boundary
  test is added.
- The O3 hostile helper proves the structural bypass is closed: its protected
  body leaf retains a subject-only bridge to the requirement occurrence, and an
  ordinary call cannot rely on the callee's old runtime prologue. The frozen
  later-gate rewalk must show that a caller claim/check proves the goal only in
  full state, while a real value branch proves it in unasserted state; stage 7
  still accepts by full-state semantics and does not prematurely emit PRV-3.
  Two-hop, clause-local-transform, recursive, and mutually recursive wrapper
  canaries must converge to the nearest acceptance-bearing requirement while a
  cycle with no protected-leaf seed remains empty.
  The three real DEFLATE requirement calls must prove the bridged goal from
  existing unasserted allocation facts with no new claim or source
  restructuring, while the retained distance claim remains unchanged.
- Every protected disposition above is compared before/after by source,
  manifest row, verdict, rule, status, and runtime result. Base64 vectors and
  output-capacity behavior remain exact. In particular,
  `fn8-neg-requires-noncopy-local` and
  `fn8-neg-requires-noncopy-cvt-local` remain FN-8 rejections; alpha expansion
  does not relax the copy-only clause-local rule. No unsupported compiler
  behavior is rewritten as source rejection.
- Recompute exact v0.26 and v0.25-archive digests, verify both native grammar
  paths and generated data, run focused semantic/lowering/backend suites,
  `make -C compiler check`, `make check`, the complete ignored adapter, the
  frozen obligation acceptance, and MCTS lint. Facts-off and ordinary modes
  must have identical source acceptance and required runtime behavior.

### Accept and stop

Stage 7 is terminal only when exact-approved v0.26 is active, ordinary calls
statically discharge one complete goal, callee bodies use the axiom without an
entry check, both real process entries preserve failure behavior, current FN-8
declaration forms and named real programs remain supported, O3 has no helper
bypass, and no unreviewed protected drift remains. If one finite atomic goal
cannot preserve those boundaries without Boolean decomposition, a duplicate
body/check path, a fake foreign adapter, or a new general theorem prover, record
the smallest reproducer as a blocker rather than narrowing the contract
silently. A positive closure advances to provenance activation; it does not
activate that gate in the same batch.

## Owner-selected roll-forward — dependency map, not execution authority

Before each later slice begins, reread its evidence, make it the sole current
step in a replacement plan, and preserve the normal candidate, impact,
owner-explanation, exact-approval, activation, and real-program loop. A
negative prerequisite measurement returns for owner disposition; it is not
permission to weaken a feature or skip ahead.

### 5b — provenance gate activation

Task 0046 supplies the positive held rule review, but only after stage 7 closes
the `requires` bypass may a replacement plan propose provenance propagation and
the signature relation. A constrained subject classified external by that
explicit-dataflow policy must then be handled by a value branch rather than
hidden in an aborting claim. Calls and real adapters must preserve the policy
through one ordinary semantic path.

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
meaning must be transitive across ordinary calls and real generated adapters,
and must specify how direct `claim`, ordinary trapping `check`, and callees that
can claim are treated; otherwise the partition is trivially bypassable.
Ordinary code retains the existing claim lifecycle, while the strict partition
requires each covered obligation to prove or take a value branch. This is not
global law and does not eliminate true-but-unprovable residue from the
no-search checker.

## Stable specification rule

The active specification stays at `spec/kernel-spec.md`. v0.24 is the current
immutable outgoing archive and v0.25 has no versioned sibling while active.
Every later candidate edits the stable file on its task branch and is reviewed
as a diff plus complete digest. Its approved atomic activation first creates
the outgoing flat archive `spec/kernel-spec-vN.md`, failing if that path exists,
and installs the new approved bytes at the unchanged stable path. The active
specification is never renamed to follow its version; archived files are never
edited, renamed, or deleted.

## Cross-stage invariants

- One normal semantic and lowering path; no program-, corpus-, function-, or
  test-shaped behavior.
- A fact widens discharge only when normative entailment derives it. Required
  checks remain unless proof discharges their exact obligation.
- Expected or externally caused failure is a value path; a claim is reserved
  for a broken program invariant and remains an executed runtime check.
- Protected conformance expectations never change without explicit owner
  approval. Unsupported capability never becomes source rejection.
- Each activated slice restores the complete gate and reruns the real consumer
  that earned it before the next slice begins.
- Durable decisions and rejected alternatives stay synchronized through the
  `mcts-mem-use` workflow; task records carry progress, not authority.

## Direction completion boundary

Wfgrep remains parked until stages 5b, 8a, 8b, 9a, and 9b above are implemented
end to end, covered by positive, negative, near-miss, and invalidation evidence,
exercised by their named real programs, and recorded in the outline; the
complete repository gate is green; and remaining claims and unsupported gaps
are reported honestly. A reproduced prerequisite blocker returned for owner
disposition is the only earlier stop. This boundary is narrower than making
`claim` the language's universal sole trap source.

## Explicitly outside the current step and selected boundary

O11 Boolean-composition precision, general loop induction, arithmetic-term
entailment or arithmetic-mode dissolution, struct/witness invariants, the OWN-3
predicate widening, and move-on-copy generic policy are not hidden parts of
this sequence. No wfgrep profiling, optimization, traversal, parallelism, or
new system-family run occurs while this direction is active.
