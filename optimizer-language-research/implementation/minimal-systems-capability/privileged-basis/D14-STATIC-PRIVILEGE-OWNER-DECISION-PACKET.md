# D14 static privilege owner decision packet

Status: research complete for owner review, 2026-07-15. No decision below is
preselected by the repository. This packet requests no production language,
specification, compiler, verifier, runtime, standard-library, container,
fact-channel, migration, or default-teaching change.

## Executive result

The production-language census, architectural comparison, hostile reviews,
minimal-basis derivation, 49-row capability crosswalk, 26-domain routing, and
structural-cost packet support one static research direction:

- use a **sealed compiler-embedded primitive registry** as the sole
  privilege-definition route;
- expose three fixed tag-free storage leaves, six checked ordinary mechanisms,
  and independently counted exact machine/runtime leaves;
- keep all containers, allocator policy, synchronization policy, async policy,
  buffering, retry, parsing, and other high-level facilities in ordinary
  checked libraries; and
- admit no source attribute, package/path convention, command-line mode,
  plugin, arbitrary contract, syscall descriptor, opcode descriptor, or
  writer-accessible unsafe route.

The evidence does not support production adoption yet. Exact proof judgments,
weak-memory/reclamation semantics, external/target row inventories, final-image
validation, 340 dense D-2 obligations across 150 contexts, and P-1 structural
parity remain open.

## Decision 1: static privilege-definition mechanism

**Question:** Select the sealed compiler-embedded primitive registry as the D14
mechanism direction for further design research?

**Recommendation:** Yes, for research selection only.

Why:

- compiler-created declaration identity, not spelling, is the authority;
- ordinary code can call and wrap fixed checked declarations but cannot define
  one;
- direct lowering and runtime helpers attach through the same identity and do
  not create another authority route;
- no shipped standard library is required;
- direct-lowered operations have no inherent runtime transition;
- one ordinary checked call model is simpler for weak writers; and
- compared with Rust, it retains compiler-owned intrinsic semantics and direct
  code shape while removing writer-accessible `unsafe`, lang-item/intrinsic
  definition, and internal-attribute routes.

What this decision does not authorize: registry schema, spelling, parser or
resolver work, compiler implementation, runtime binding, or any primitive row.

## Decision 2: abstract public capability basis

**Question:** Accept the following basis as the hypothesis for exact design and
falsification work?

**Recommendation:** Yes, with all recorded gaps fail-closed.

Fixed storage leaves:

1. checked tag-free carrier/place formation and structural layout;
2. exact `Vacant<T>`/`Live<T>` put, take, and destroy transitions; and
3. recoverable empty physical-root acquisition and exact empty release.

Checked ordinary mechanisms, available to every unprivileged library:

1. opaque generative resource sealing under fixed checker rules;
2. exact transition closure and statically checked disposition;
3. per-leaf borrow-source maps and scoped disjointness;
4. static generic/stateful callable contracts;
5. checked sharing/interference invariants under one fixed memory model; and
6. checked refinement sealing and invalidation.

Independently counted machine/runtime leaves:

- stability/external-access leases;
- typed atomic events;
- thread runtime events;
- exact external resource events, one row per semantic event;
- exact target/device events, one row per semantic event; and
- executable-code validation, activation, lease, call, and unload.

Why this is not a universal back door:

- ordinary proofs cannot add semantics, effects, facts, proof rules, lowerings,
  or foreign assumptions;
- P7/P8 are sets of exact compiler-owned rows, not generic syscall/opcode/
  contract descriptors; and
- replacement, swap, relocation, resize, named containers, and all policy stay
  ordinary derived code.

What this decision does not authorize: source syntax, proof language, memory
model, exact event inventory, implementation, or a completeness claim.

## Decision 3: bounded validation authorization

No experiment is currently authorized. If Decisions 1 and 2 are accepted, the
smallest evidence sequence is:

### 3A. Deterministic safety-model slice

Formalize only P1/P2 plus Q1/Q2 for a dense prefix, a two-interval gap, and one
inline-to-heap representation transition. Mutation-test dead reads, duplicate
owners, wrong drops, abandoned holes, forged state, failure before/after commit,
and ZST identity.

**Recommendation:** authorize first. This can falsify the foundation before
code-shape work.

### 3B. Structural no-tax slice

Only after 3A is green, build a detached prototype for the existing fixed full
buffer control, dense affine sequence, and inline-small spill. Compare exact
layout, allocation calls, initialized/zeroed bytes, moves, drops, branches,
checks, code size, and optimized bodies against frozen native baselines. Do not
run throughput timing until structural parity passes.

**Recommendation:** authorize conditionally on 3A's hostile-reviewed PASS.

### 3C. Exact external-row slice

Specify one partial byte-read row that forbids retention, one asynchronous row
that retains a P4 buffer lease, and one close row. Adversarially test partial
progress, interruption, cancellation, lifetime, exact event counts, and fact
attenuation.

**Recommendation:** authorize after 3A, independently of 3B. Do not expand into
a broad FFI/OS catalog.

Q5 concurrency, P9 JIT, MMIO, GPU, package, or standard-library work should
remain blocked until these smaller slices show whether the basis survives.

## Decision 4: production work

**Question:** Authorize a language/specification/compiler/runtime production
change now?

**Recommendation:** No. The research is not sufficient for this decision, so no
production approval is requested.

Any later production request must separately identify exact rules, registry
rows, fact channels, target support, negative tests, migration impact, P0/W1/W3
deltas, and verification gates.

## Evidence packet

- `STATIC-PRIVILEGE-MECHANISM-CENSUS.md` — Rust, Swift, Go, and .NET primary-
  source mechanism evidence.
- `STATIC-PRIVILEGE-GATE-DECISION.md` and
  `STATIC-PRIVILEGE-GATE-HOSTILE-REVIEW.md` — mechanism comparison and attack
  review.
- `MINIMAL-STATIC-PUBLIC-CAPABILITY-BASIS.md`, its hostile review, and the
  49-row crosswalk — basis, removal witnesses, gaps, and exact accounting.
- `STATIC-BASIS-DERIVABILITY-COST-PACKET.md` and its hostile review — ordinary-
  library routes, 26-domain map, structural costs, held-outs, and bounded
  validation proposals.

Historical F/C/S, signature, snapshot, replay, revocation, and identity-graph
reports answer a different third-party authorization problem. No owner decision
on that branch is requested.

## Suggested owner response surface

The owner can respond with four independent dispositions:

1. mechanism direction: select / reject / revise;
2. abstract basis hypothesis: accept / reject / revise;
3. validation: authorize 3A only, authorize the conditional 3A-to-3B/3C
   sequence, or authorize none; and
4. production work: no authorization requested.
