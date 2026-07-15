# Dense Unique-Owner Family Lock A

Status: `OWNER_REVIEW_READY_RESEARCH_ONLY`, revision `F-DENSE-LOCK-A-R5`.
Frozen at: `2026-07-15T01:11:35-07:00` (`America/Los_Angeles`).

This dossier completes the research and preregistration authorized by D13. It
does not select a language design, construct a candidate, grant Candidate
Freeze B, expose held-out material, execute or score a candidate, restart E0.1,
or authorize compiler or production work. Those actions require later explicit
owner authorization.

Integrating author task identity: `/root`. Final independent exact-byte
reviewer task identity: `/root/whole_lock_hostile`; that reviewer edits none of
the reviewed lock bytes. Coverage and contract-soundness personal/task
identities were not retained, so their durable identities are the exact review
artifacts: `COVERAGE-REVIEW-R3@d8ee4c161e84a3996c0167b54576893074a16775b30994ab8236e79fa63d4798`
and `CONTRACT-SOUNDNESS-REVIEW-R1@20b6325366c961a5d608066da8acd9a9c19352290fdaa44e3666f2e14430c7c7`.
The retained performance reviewer task identity is
`/root/repair_soundness_protocol`, with durable identity
`PERFORMANCE-REVIEW-V5@e42823c8ecf94b2ac5c898c3215c511e9881fd082b7b77a112e98ff3b3b7bfe1`.
The coverage independence claim is bound to the D13-R3 decision-gate entry at
commit `32c01e188ba55f652700cf8547187fe462302f0b`, entry SHA-256
`08b5322c032d878f1f2ac2055095d6c14d71756320d5a9ef3ddd8121232e7be2`.
The soundness and performance reports themselves record independent review and
attest that their reviewers edited no reviewed bytes. Coverage-review personal,
task, and no-edit provenance was not retained, so this lock makes no stronger
coverage no-edit claim than the exact durable evidence above.

The required predecessor lock set is empty. This is the first owning-storage
Family Lock. G0-Core is a frozen research input, not a completed family
predecessor: closing commit
`a4de0eb70c345dcd198b11f435a5538ccc863113`, gate heading
`G0-Core capability accounting is complete; mechanisms remain unselected
(2026-07-14)`, and 110-artifact manifest SHA-256
`f0eced756688affef1732a133c43fb39ab6fc672334dca27b26129ddb5123719`.
Cross-family state or fact exposure: `NONE`. Candidate partial states and fact
channels remain sealed and unselected; a later family cannot import dense
capability until a separate production-adoption decision closes it.

## 1. Owner result

The dense unique-owner gap is real. Current xlang cannot let an ordinary
checked library implement a contiguous growable sequence of arbitrary
region-free, borrow-free affine values with spare capacity, exact move-out,
direct relocation, live-prefix destruction, and failure-atomic growth. The
same missing foundation blocks efficient fixed AoS records, stacks, sorting,
compaction, small sequences, gap buffers, and the dense predecessor required by
later sparse, heap, ordered, identity, arena, text, and iteration families.

The research found no defensible single mechanism to select by argument alone.
It froze five pairwise-distinct candidate-mechanism contracts, one common experimental
substrate, 303 exact caller contracts, a mathematical ownership oracle, and an
exact performance protocol against same-shape Rust 1.97 references. Every
candidate must preserve the same public behavior, ownership, failure, layout,
algorithm, growth, allocator, and protected-baseline obligations. The only
variable is how an operation-local partial dense state is represented and
closed.

The lock is ready for six owner decisions. Even if all recommended choices are
accepted, candidate construction remains unauthorized until the operational
identities and custody inputs are frozen, a separate authorization is given,
and the authorized reference-only pilot closes as feasible.

## 2. Bounded claim

A later successful experiment may establish only this claim:

> Ordinary no-unsafe xlang libraries can implement the frozen sequential,
> unique-owner dense-prefix contracts for region-free, borrow-free,
> non-address-sensitive affine payloads with the frozen ownership, failure,
> destruction, asymptotic, structural, and target-local performance bounds,
> while protected B-FIX and B-P2 paths remain unchanged.

The lock does not claim borrow-bearing or region-bearing payloads, nonlocal
lifetime storage, shared ownership, dynamic borrowing, concurrency, pinning or
address-sensitive values, custom allocators, FFI resources, async cancellation,
panic unwind, exception handling, recoverable cleanup after traps, complete
text semantics, target intrinsics, platforms other than the selected exact
native targets, a universal platform or architecture result, cyclic
collection, or a complete standard library. Those remain named later families
or explicit non-claims. W-PIPE also remains owned by the later iteration
family.

## 3. Exact audit closure

The coverage authority derives its domain from the frozen G0 evidence rather
than from a candidate-facing API list. Independent hostile reconstruction
confirmed:

- 65 audit clusters;
- 426 selector children;
- 1,400 exact evidence-to-target terminals;
- 780 evidence/member bindings;
- 101 overlay bindings;
- 85 role bindings;
- 1,267 capability bindings; and
- all 456 direct evidence identities anchored exactly once.

Five raw identities cross topology boundaries and correctly retain their real
gate-local excluded outcomes without inheriting dense applicability. Twenty-two
mutation attacks reject rerouting, substitution, omission, and coherent
authority forgery. The exact reviewed bytes and the disposition of both failed
earlier drafts are in
[`DENSE-COVERAGE-HOSTILE-REVIEW-ROUND-3-PASS.md`](DENSE-COVERAGE-HOSTILE-REVIEW-ROUND-3-PASS.md).

## 4. Exact contract and ownership closure

The contract registry contains 303 exact member/outcome units.
All 5 candidates bind to all 303 units, producing exactly 1,515 candidate/contract
bindings. Ninety-seven adapter groups require identical observable results,
owners, allocation roots, borrows, facts, and failure behavior across all
arms. The model includes:

- exact initialization, move-out, relocation, replacement, swap, destruction,
  allocation transfer, and release events;
- checked capacity arithmetic and allocation commitment points;
- exact success, pre-abort, recoverable-failure, and static-rejection outcomes;
- shared, unique, and owning traversal;
- stable and unstable sort families, callable state, and call order;
- all four active stored-borrow routes without weakening their provenance;
- zero-sized affine ownership and destruction; and
- eight research-only optimizer-fact contracts with exact owner, root, version,
  producer, consumer, invalidation, transfer, and facts-off rules.

Affinity alone does not complete a protocol because an affine value may be
abandoned. Every normal exit must therefore leave one complete valid owner,
prove exact consumption, or invoke separately specified compiler-derived typed
repair. A conventional `finish` call and writer discipline do not count.
Traps do not unwind and cannot perform cleanup.

The executable oracle generated 2,002 deterministic traces.
Across those traces, 262 are hostile cases. It rejected 32 registered mutations and three
independently constructed coherent attacks,
including owner minting or loss, stale facts, live-prefix facts in Hole state,
candidate-private cursors, wrong allocation roots, wrong call order, invalid
ZST behavior, and execution-authority escalation. The independent result is in
[`DENSE-SOUNDNESS-HOSTILE-REVIEW-PASS.md`](DENSE-SOUNDNESS-HOSTILE-REVIEW-PASS.md).

## 5. Historical E0.1 reconciliation

The lock explicitly disposes all 13 rows required by the historical E0.1
traceability record and the Family Lock template, and classifies all 93 unique
exact members. The frozen input is
`E01-TRACEABILITY.md` at SHA-256
`2973109ddfee2b6caf8f0b4eedbbbfc55ec933e12c842735dd76c490f106e613`.
The machine-checked result is
[`DENSE-E01-DISPOSITION-AUTHORITY.tsv`](DENSE-E01-DISPOSITION-AUTHORITY.tsv).
Its exact per-member authority is
[`DENSE-E01-MEMBER-CLASSIFICATION.tsv`](DENSE-E01-MEMBER-CLASSIFICATION.tsv):
84 new mandatory exact contracts, 6 raw or initialization-authority
rejection-evidence members, and 3 lazy lifecycle evidence members that remain
OD-4 evidence rather than mandatory operations.

| Historical input | Exact disposition in this lock |
| --- | --- |
| Current fixed buffer | Retained as the protected `B-FIX` identity control. |
| Declarative Copy | Superseded as a dense mechanism; the production Copy choice remains open. |
| Affine fixed builder | Retained as historical exact-fill evidence; superseded as a general dense route. |
| Automatic structural Copy | Not selected; the production alternative remains open. |
| Copy by default plus negative `affine` | The fail-closed rejection is preserved. |
| Affine fixed-storage predicate | Revised into target-derived storage eligibility without semantic Copy. |
| Recursive or single-level recipe | Superseded by explicit callable construction contracts; no grammar exception is assumed. |
| Explicit Repeat/Clone | Retained as a separate semantic duplication contract, never relocation. |
| Per-slot builder/initialized prefix | Revised: `Dense[0,len)` is steady state and candidate partial states are operation-local. |
| Public raw or split uninitialized privilege | The ordinary-writer rejection is preserved. |
| Layout and capacity controls | Retained and expanded into exact target, growth, arithmetic, and protected fixtures. |
| Numeric and statistical inputs | The 1.02/0.90/0.85 margins remain; inference and power are replaced by the frozen exact protocol. |
| Adoption, xlc migration, and teaching | The three decisions remain separate and unauthorized. |

Fixed AoS layout, semantic Copy, and partially live affine storage are three
orthogonal questions. This lock admits fixed AoS as a storage shape, never
infers Copy from it, and compares only five ways to close operation-local
partial affine state. The following prose names only non-exhaustive categories
and examples, not the authoritative enumeration: unknown-length append, pop,
ordered and unordered remove, growth, relocation, truncate, clear, stable
compaction, stable and unstable sort, clone and clone-from, owning traversal,
exact live-prefix destruction, recoverable reserve, dynamic disjoint mutation,
result reborrows, and ordinary-library H-FLATSET. The exact exhaustive
84-operation enumeration is the member-classification TSV and is bound into
E0.1 disposition row 09. The detached E0.1 candidate and historical
measurements remain inactive evidence; this reconciliation does not restart
them.

## 6. Common experimental substrate

A fair comparison needs one identical substrate in every arm. The recommended
experiment-only substrate supplies:

- erasable user-defined representation sealing;
- the existing generic monomorphization and reborrow/result-provenance rules;
- direct effectful behavior calls with exact retained-state ownership;
- one checked allocation-owner facade; and
- one affine single-live-interval owner for whole-sequence owning traversal.

The allocation facade transfers one exact block owner and checked failure
results. It grants no raw bytes, writer-set liveness, unchecked capacity
mutation, manual deallocation, or forged provenance. The interval owner holds
one master allocation and one live range `[front, back)`, kills an endpoint
owner before yielding it, destroys the exact remainder on abandonment, releases
once, and uses logical indices for ZST identity. It exposes no hole, arbitrary
liveness, repair-to-dense operation, or second range.

This substrate is a candidate-neutral experimental control, not a production
selection. Any semantic or cost change to it reopens every candidate binding
and protected baseline.

## 7. Finite candidate set

| Candidate | Where partial-state authority lives | Principal cost or risk |
| --- | --- | --- |
| `C-ATOMIC-TRANSITIONS` | A sealed lexical range-transition region exposes checked ownership events and closes every normal escape. | Region surface and callback-spanning validation may be large. |
| `C-LINEAR-REBUILD` | An exact-use rebuild scope owns source, destination, allocation, proof, and moved values. | Adds a second exact-use ownership mode and pervasive flow obligations. |
| `C-DERIVED-REPAIR` | An affine transition scope may hold a hole; the compiler derives typed repair on every normal abandonment edge. | Enlarges kernel semantics, cold code, and the soundness TCB. |
| `C-PROOF-CARRYING-STATE` | Checked splits produce zero to two statically proved live ranges under one allocation owner. | Requires split ownership, range provenance, rejoin, and proof erasure. |
| `C-RUNTIME-TOPOLOGY` | A sealed owner carries compact O(1) `Dense` or one-`Hole` topology state. | Creates a hostile-reviewed runtime fact channel and must prove zero steady-dense tax. |

The arms are pairwise separated by lifecycle, proof location, runtime metadata,
normal-exit handling, TCB, source surface, and code shape. Candidate-private
allocators, cursors, growth policies, sealing, result provenance, or standard-
library privilege are forbidden. A per-slot tag or bitmap is not the runtime
topology arm and is structurally rejected for the dense route.

Rejected alternatives include writer-visible raw or uninitialized memory,
`set_len`, dummy/default full-capacity initialization, an affine `finish`
convention, whole-sequence rebuild for each mutation, per-item heap allocation,
privileged standard-library-only storage, and a general user finalizer.

## 8. Required capability surface

Every surviving arm must derive the exact contracts for fixed AoS storage,
unknown-length append, affine push and pop, reserve and growth, shrink,
ordered insert/remove, swap-remove, dynamic swap, truncate, clear, clone,
clone-from, append-by-move, retain, eager removal and splice, stable and
unstable sort, stack use, shared/unique/owning traversal, and the applicable
view and conversion operations.

The lock also includes three ordinary-library generativity witnesses:

- `W-SMALL`: inline-to-heap small sequence;
- `W-GAP`: gap-buffer editing; and
- `H-FLATSET`: a sorted affine flat set implemented without importing a
  completed dense container.

`B-FIX` and `B-P2` are protected no-tax controls. A candidate fails if it adds
fields, branches, checks, allocations, code paths, payload traffic, or optimizer
dependencies to those default shapes.

## 9. Reference algorithms and structural ceilings

All candidates use the same frozen algorithms and Rust 1.97 growth policy.
Examples of exact ceilings include:

- no-grow push performs zero allocation and one initialization;
- no-grow insert directly relocates exactly the suffix, not repeated swaps;
- ordered remove moves out once and directly relocates the remaining suffix;
- swap-remove performs at most one last-element relocation;
- retain calls the predicate once per original element and relocates each
  retained post-hole element at most once;
- clear, truncate, and destruction touch exactly the live values, not capacity;
- traversal is one pass with no intermediate allocation or indirect behavior
  call; and
- H-FLATSET lookup remains logarithmic with no per-slot occupancy metadata.

No candidate may silently require `Copy`, `Clone`, `Default`, a dummy value, a
second persistent payload copy, or a different asymptotic algorithm. Reference
routes and structural counters are exact per operation and workload shape.

## 10. Frozen performance protocol

The performance registry derives from all 303 contracts and contains:

- 137 `TIMED_PRIMARY`, 144 `FUNCTIONAL_ONLY`, 13 `STRUCTURAL_ONLY`, and 9
  `EXCLUDED` dispositions;
- 97 standalone same-shape Rust operation gates;
- 520 exact matrix cells, including 502 primary timed cells;
- explicit payload separators for `U8`, `U64`, `ROW24`, `ROW56`, `AFFINE24`,
  `AFFINE64`, `AFFINE256`, `BEHAVIOR`, and `ZST`;
- AArch64, x86-64, and i686 structural checks for protected and witness shapes;
  and
- 27 explicit operational blockers rather than invented defaults.

Operations, sort shapes, edit positions, retain ratios, clone-from length
relations, splice shapes, traversal modes, payloads, sizes, and targets are
never pooled. Each selected native target and each exact primary cell must pass
its own candidate/Rust noninferiority gate at ratio 1.02. Candidate/candidate
dominance additionally requires a registered time or positive-reference memory
benefit under one global multiplicity family. Zero-reference memory is a
structural equality route and can never create a benefit.

The design uses six byte-identical Rust pseudo-treatments, a balanced Williams
schedule, five inert layout salts, fresh child processes, and only
reference-derived joint crossed-cycle supports for planning power. Primary
inference uses strict raw-integer candidate/reference comparisons at every
scheduled slot: no logarithm, fitted adjustment, covariance estimate, residual,
or floating-point threshold enters a decision. The conditional null bounds the
equal-weight scheduled mixture of heterogeneous slot-success probabilities.
Exact worst-case Poisson-binomial tails reduce its least-favorable decision to
the registered fair-binomial upper tail, while exact whole-cycle empirical
dynamic programming preserves the dependence observed within a reference pilot
cycle for planning power. Period and predecessor summaries remain descriptive
and never adjust inference or power. The exact comparison cross-products,
benefit nulls, ties, Holm step-down order, noninferiority decisions, clustered
planning alternatives, and infeasibility rule are machine-validated. Planning
power is conditional on the frozen reference-empirical model and does not claim
to know candidate-specific variance or tails. Candidate data cannot tune sample
size. The selection set
`S` contains candidates that pass every structural and Rust-floor gate and
dominate all four other candidates; selection occurs only when `|S| = 1`.
Every other cardinality returns `NO-SELECTION`.

The exact protocol and its rejected predecessor freezes are independently
reviewed in
[`DENSE-PERFORMANCE-HOSTILE-REVIEW-PASS.md`](DENSE-PERFORMANCE-HOSTILE-REVIEW-PASS.md).
No pilot or candidate observation has occurred.

## 11. Six owner decisions

The executable protocol keeps every option explicit. The recommendations below
optimize for a general-purpose systems language, a clean first comparison, and
no tax on default code.

### OD-0: Common substrate or separate prerequisite locks

Recommended: `OD-0-COMMON-EXPERIMENTAL-SUBSTRATE`.

Use the exact candidate-neutral substrate in all five arms without selecting a
production spelling or implementation. This lets the dense mechanisms be
measured now while charging every shared cost equally. The alternative is to
close sealing, calls, allocation ownership, reborrow/provenance, and owning
intervals as separate production-relevant locks first; that gives sequential
attribution but postpones all dense candidate work and requires the dossier to
be regenerated after those locks close.

### OD-1: Recoverable allocation failure surface

Recommended: `OD-1-RESERVE-FIRST`.

Only `try_reserve`-like preparation returns arithmetic or allocation failure.
A caller needing recovery reserves first and then uses a no-grow mutator.
Default push, insert, append, resize, collect, and clone retain checked
arithmetic and the current divergent OOM boundary without a recoverable-result
branch. The alternative returns the unchanged container and all offered affine
owners from every growth-capable mutator, at the cost of a larger ABI and a
mandatory fast-path branch.

### OD-2: Native target scope

Recommended: `OD-2-DUAL-NATIVE`.

Require one exactly identified AArch64 macOS runner and one independently
identified x86-64 Linux runner; both must pass every intersection gate. Neither
runner identity exists yet. This supports only a two-target claim, not universal
architecture independence. The alternative is Mac-only measurement explicitly
labeled target-local, with no architecture-general production claim.

### OD-3: Zero-sized affine payloads

Recommended: `OD-3-INCLUDE-ZST`.

Include ZST values with logical owner identity, checked length, logical
capacity `usize::MAX`, zero payload allocation, and exactly `len` destructions.
This tests that ownership and disjointness do not depend on addresses and adds
no allocator-byte benefit. The alternative narrows the first family claim to
positive-size payloads and requires generic libraries to reject or defer ZST.

### OD-4: Removal-consumption contract

Recommended: `OD-4-EAGER-AND-SCOPED-CONSUME`.

Require eager owning-result removal plus a nonescaping scoped consume/fold
operation. The scoped form moves each removed owner once in source order,
allows O(1)-state discard or online consumption, allocates no result sequence
unless the caller collects, and repairs the dense owner before every normal
return. It introduces no persistent repair-bearing cursor. Eager-only is
simpler but forces an avoidable O(k) result allocation for discard/streaming
consumers. Promoting Rust-style lazy drain/extract/splice would require a new
persistent lifecycle and must reopen the lock.

### OD-5: Compile-time crossover

Recommended: `OD-5-NO-CROSSOVER`.

Make each of the five mechanisms cover the complete frozen matrix and select
one mechanism or `NO-SELECTION`. This keeps the first experiment attributable.
The alternative adds a fully enumerated sixth static crossover candidate, which
changes treatment count, schedule, power, multiplicity, META-5 accounting, and
every cell assignment; it therefore requires a revised lock before any
construction.

The complete option text and consequences are executable in
[`dense_owner_decisions.py`](dense_owner_decisions.py). None of the six
recommendations is silently selected by this dossier.

## 12. Remaining operational blockers

After the owner resolves OD-0 through OD-5, the next research-stage revision still
needs exact candidate-author and service identities, equal resource budgets,
disclosure authority, repair rules, common-substrate artifacts, allocator and
harness identities, native runner and counter identities, randomization
custody, power-engine resource limits, layout evidence, build protocols, and
the required W/H fixture custody. It also requires one exact common repository
baseline and keeps the OD-4 reference adapter separate from the later
candidate-side META-5/compiler artifacts. The performance registry names all
27 blockers. Each Mac-local branch has 8 direct reference-pilot prerequisites
and 21 cumulative construction prerequisites; each dual-native branch has 9
and 22 because the x86 runner is additional. Reference-pilot observations are
deliberately not a prerequisite to
granting the next bounded authorization: they cannot exist until that
authorization permits the reference-only campaign. They are nevertheless a
hard execution prerequisite to candidate construction. The authorization may
permit the pilot and contingent construction in sequence, but it cannot permit
the first candidate prompt or source construction before pilot closure.

After that authorization, the reference-only pilot must freeze its raw rows,
assignment manifest, positive-memory eligibility, whole-cycle support, and
selected fixed sample count. Only a feasible result permits candidate
construction to start. Those pilot artifacts remain frozen before Candidate
Freeze B and any candidate-primary execution. Pilot results cannot change
contracts, candidates, algorithms, endpoints, benefit margins, or the
registered alternatives.

Inability to construct an arm or pass the frozen protocol is a mechanism result;
it does not permit a new author, mechanism, budget, tuning round, target, or
post-result crossover. Any semantic input change reopens exact-hash review.

## 13. What the next authorization would mean

If the owner accepts or replaces all six decisions, the lock is regenerated for
that one branch and receives fresh exact-hash hostile review. A separate owner
authorization may then permit the reference-only pilot and candidate
construction contingent on a feasible pilot closure, in that order. Candidate
Freeze B, scored execution, held-out access, language or specification
selection, compiler implementation, production adoption, E0.1 restart, xlc
migration, and default teaching remain later gates.

The measured outcome may be one candidate or `NO-SELECTION`. Only a selected
candidate that passes every contract, soundness, structural, protected,
same-shape Rust, and statistical gate can return for a production language-
design decision. Research approval never implies production adoption.

Approving this lock grants only the six owner research-protocol decisions. It
does not authorize a reference pilot, candidate construction, Candidate Freeze
B, candidate or held-out execution, held-out access, candidate selection or
scoring, a language or specification change or decision, compiler
implementation, production implementation or adoption, E0.1 restart, xlc
migration, or default teaching. Every such action requires a later separate
authorization.

## 14. Authorization and completion boundary

| Owner boundary | Exact record |
| --- | --- |
| Activities authorized by approving this lock | Resolve the six owner research-protocol decisions only. |
| Candidate Freeze B | Requires a separate owner authorization after construction, closure of every applicable cumulative Freeze B blocker, and fresh independent exact-hash construction review. |
| Scored and held-out execution | Requires a separate owner authorization after Freeze B pins every candidate hash and an independent custody audit freezes source, tests, traces, disclosure, access, leak, and rotation records. |
| Family closure | One selected branch must close all 303 exact contracts, every applicable M/W/H role and protected B control, proof, structural and measured Rust-floor gates, fact reports, and reviews; `NO-SELECTION` does not close the family. |
| Later gates | Production adoption, specification/compiler work, xlc migration, default teaching, complete-floor, and whole-language claims remain separate or prohibited. |

The lock-completion record is deliberately mixed PASS/BLOCKED. `PASS` means a
research authority or protocol is frozen; it does not mean the corresponding
candidate, execution, or production gate exists.

| Completion item | Current exact status |
| --- | --- |
| All field markers resolved | `PASS_RESEARCH_INSTANTIATION` |
| Contract/capability/witness closure | `PASS_RESEARCH_FREEZE_FAMILY_CLOSURE_NOT_CLAIMED`; W-SMALL, W-GAP, and H-FLATSET fixtures/custody remain blocked. |
| Soundness fixture freeze | `PASS_EXACT_BYTES_CONTRACT_SOUNDNESS_ONLY` |
| Construction/correction freeze | `BLOCKED_BEFORE_FIRST_CANDIDATE_PROMPT`; the exact cumulative blocker set is in the stage manifest. |
| Performance and selection freeze | `PASS_PROTOCOL_ONLY_EXECUTION_AND_SELECTION_BLOCKED` |
| B-FIX/B-P2 no-tax oracle freeze | `PASS_PROTOCOL_ONLY_CANDIDATE_COMPARISON_BLOCKED` |
| Held-out custody freeze | `BLOCKED_NO_HIDDEN_ARTIFACT_CLAIMED` |
| META-5 closure | `PASS_RESEARCH_DELTAS_UNSELECTED_PRODUCTION_ARTIFACTS_BLOCKED` |
| E0.1 disposition | `PASS_13_INPUTS_93_MEMBERS_NO_RESTART` |
| Independent hostile reviews | Layer passes are frozen; external exact-byte whole-lock review remains required at manifest construction time. |
| Repository verification | Required on final bytes before and after external review. |
| Owner authorization | `BLOCKED_SIX_OWNER_DECISIONS_UNRESOLVED` |
| Durability | Final commit and append-only decision-gate entry remain external to the self-hashed manifest. |

The manifest additionally joins each required artifact class to exact paths and
hashes, its producer, reviewer, and status. H-FLATSET custody is represented by
`PENDING_EXTERNAL_H_FLATSET_CUSTODY`; no hidden source or hash is claimed.

## 15. Durable evidence map

- Coverage closure:
  [`DENSE-COVERAGE-HOSTILE-REVIEW-ROUND-3-PASS.md`](DENSE-COVERAGE-HOSTILE-REVIEW-ROUND-3-PASS.md)
- Contract and soundness closure:
  [`DENSE-SOUNDNESS-HOSTILE-REVIEW-PASS.md`](DENSE-SOUNDNESS-HOSTILE-REVIEW-PASS.md)
- Performance protocol:
  [`DENSE-PERFORMANCE-PROTOCOL.md`](DENSE-PERFORMANCE-PROTOCOL.md)
- Performance hostile review:
  [`DENSE-PERFORMANCE-HOSTILE-REVIEW-PASS.md`](DENSE-PERFORMANCE-HOSTILE-REVIEW-PASS.md)
- Historical E0.1 member classification:
  [`DENSE-E01-MEMBER-CLASSIFICATION.tsv`](DENSE-E01-MEMBER-CLASSIFICATION.tsv)
- Exact whole-dossier manifest:
  [`DENSE-LOCK-ARTIFACT-MANIFEST.json`](DENSE-LOCK-ARTIFACT-MANIFEST.json)
- Executable whole-dossier verifier:
  [`verify_dense_lock.py`](verify_dense_lock.py)

The closing manifest hashes every canonical artifact and the controlling
repository records without including its own external review, avoiding a hash
cycle. The external whole-lock review pins the manifest, dossier, builder, and
verifier exact bytes. Repository gates remain mandatory; design-memory lint is
required whenever the tree itself changes.
