# Privileged Basis Completion Lock

Status: D14 research acceptance contract, 2026-07-15. This lock defines what
must be true before the privileged-gate and public-basis research may claim
completion. It authorizes no implementation, experiment, language or
specification change, standard library, container construction, production
fact channel, xlc migration, E0.1 restart, or default teaching.

## 1. Claim under test

For one fixed xlang semantic root, one nonforgeable admission mechanism must
account for every privileged semantic edge. Above that gate, an ordinary
unapproved producer must be able to construct proof-checked modules that
implement the required container and systems contracts without a privileged
named container, writer-trusted assertion, hidden `Copy`/`Clone`/`Default`
premise, asymptotic regression, pathological storage, or mandatory runtime tax
on a contract that does not require the taxed state.

The claim is about expressibility and structural cost. It does not claim that
the project has implemented the mechanism or a standard library, that proof
production is automatic, or that unmeasured code has a particular throughput.

## 2. Gate acceptance rows

| ID | Requirement | Evidence required for PASS |
|---|---|---|
| G-1 | One admission predicate | Base and out-of-band entries are authorization records consumed by one exact use-time predicate. No file path, source name, attribute, flag, environment input, package identity, linker name, cache record, or receipt independently grants authority. |
| G-2 | Fixed semantic root | The reference machine semantics, proof kernel, effect/fact schema, capsule schema, canonical decoder, and approval-key policy have one immutable semantic-root identity. Replacing any of them creates a different toolchain identity rather than entering through the gate. |
| G-3 | Authenticated active state | The active authorization snapshot is canonical, signed under the root policy, monotonically versioned in protected installation state, and linked to its predecessor. Rollback, replay, mix-and-match, stale-cache, target substitution, frame substitution, program substitution, and policy-root substitution are rejected. |
| G-4 | Exact final binding | `CERTIFIED` binds the final resolved implementation or lowering bytes and complete semantic dependency cone. `TRUSTED_FOREIGN` binds either exact provider identity or one exact root-defined provider-validation policy while naming unavailable provider bytes as a residual assumption and attenuating facts. Both bind the public contract, representation, evidence, target/backend, program/frame scope, current membership, dependency cones, and review identity. Snapshot identity never substitutes for the tuple or cones and need not become every object's semantic cache identity. |
| G-5 | Revocation and revalidation | Binary libraries, incremental artifacts, LTO inputs, objects, caches, and final links revalidate exact current membership and certificate validity at use. Revocation invalidates future builds without pretending to erase previously emitted binaries. |
| G-6 | Ordinary proof boundary | A proof-bearing module is ordinary only when it uses existing public semantics and derives only within the fixed reference policy. Ordinary nominal types, checked contracts, and hidden representations require no approval. Adding a machine instruction, helper, alternative lowering, ABI/OS/device edge, foreign assumption, proof axiom, fact/effect kind, unproved primitive representation axiom, or backend behavior requires an exact admitted entry or a new semantic root. |
| G-7 | No hidden second gate | Plugins, compiler-private imports, bootstrap stages, runtime helpers, link aliases, development modes, and package managers cannot introduce semantic authority outside G-1. The unavoidable authority to replace the trusted compiler/root is named as the TCB boundary, not misreported as part of the in-language gate. |

## 3. Public checked basis rows

The basis is minimal by semantic authority, not by counting names. Combining
two independent obligations under one operation name does not remove either
obligation.

| ID | Requirement | Evidence required for PASS |
|---|---|---|
| B-1 | Generative resource root | Dynamic, inline, static, arena, mapped, and foreign origins can yield a root with exact extent, alignment, layout, provenance, version, acquisition outcome, and one disposition. Cross-root authority cannot be forged or widened. |
| B-2 | Typed cell lifecycle | The checked model distinguishes dead/uninitialized storage from one live owned `T`; initialization, move-out, replacement, permutation, relocation, destruction, and reuse conserve every affine value and never read or destroy a dead cell. |
| B-3 | Access and borrowing | Every read, write, shared borrow, unique borrow, result reborrow, cursor yield, and invalidation is tied to exact root and footprint authority. Disjointness is proved or checked, never inferred from pointer inequality. |
| B-4 | Aggregate protocol abstraction | Ordinary modules can seal representation-specific liveness and resource predicates, including runtime-dependent predicates, without making their metadata a trusted assertion. Every safety-authorizing proposition has one explicit disposition: bounded static proof, runtime validation/enforcement, or a producer certificate checked against the fixed policy when general predicates are selected. Common bounded shapes require no general proof search, and no topology-specific privilege is permitted. |
| B-5 | Exact transition scope | A temporary unpacked or invalid aggregate state cannot escape a normal edge. Swap, replace, relocation, repair, and failure-atomic commit are either derived inside an exact checked transition scope or justified as independent irreducible operations. |
| B-6 | Allocation outcomes | Abort-on-shortage and recoverable shortage have exact, distinct commitment and offered-owner dispositions. Recoverable failure leaves every precommit owner valid and does not become a contract trap. |
| B-7 | Normal abandonment | Every abandonable owner has executable, verified disposition on fallthrough, return, break, propagation, callback stop, and cancellation. A proof certificate alone is not cleanup. The selected mechanism covers nested and recursive resources, exact effects, termination, and external-resource abandonment without user-trusted finalizers. |
| B-8 | Opaque ordinary module | An ordinary producer can export a safe checked contract while hiding roots, representation, proof terms, and transition functions. The caller cannot construct or mutate sealed proof state except through verified exports. |
| B-9 | Fixed verifier boundary | Solvers, proof search, AI generation, optimizers, and derived proof rules may be untrusted. Final acceptance checks the exact artifact against a fixed reference policy; a derived rule must itself be proved to imply that policy and cannot install an axiom. |
| B-10 | Minimality | Every retained semantic authority has a separating witness showing that deleting it violates safety, resource exactness, ordinary-library generativity, or a protected cost. Every rejected alternative has an explicit dominance or hard-constraint failure. |

## 4. Derivability rows

| ID | Requirement | Evidence required for PASS |
|---|---|---|
| D-1 | Complete demand domain | All 26 rows of `SYSTEMS-DOMAIN-LEDGER.md` terminate in an exact ordinary-language derivation, an ordinary proof-checked library derivation, a toolchain-only disposition, an irreducible admitted entry, or an owner-ratified non-goal. `BASIS?`, `future logic`, and unnamed deferral are not terminal states. |
| D-2 | Exact G0 obligations | Every mandatory G0 operational obligation and every applicable exact dense Family Lock member/outcome contract has a route through the basis or an exact later-domain dependency. A coarse cluster or container name cannot discharge a member. |
| D-3 | Sequential adversaries | W-SMALL, W-GAP, H-FLATSET, H-STORE, dense growth, deque/ring, B-tree node, iterator/drain, inline storage, arbitrary sparse occupancy, and recyclable stable identity have constructive ownership and cleanup traces. |
| D-4 | Recursive and address-sensitive adversaries | Unique recursion, linked topology, multi-block arena, pin/address stability, intrusive membership, and self-reference either derive without pathological per-element allocation or end in an exact separately admitted machine/resource dependency. |
| D-5 | Shared and concurrent adversaries | Shared/weak ownership, dynamic borrowing, atomics, locks, channels, lock-free reclamation, thread lifecycle, and data-race exclusion have exact public protocol routes. Merely naming atomics or a future concurrent logic does not pass. |
| D-6 | External and asynchronous adversaries | Files, sockets, process handles, clocks, FFI, partial I/O, callbacks, async cancellation, device/MMIO, and target operations separate irreducible admitted effects from ordinary protocol and policy code, including normal and cancellation cleanup. |
| D-7 | Fresh held-outs | At least three structures not used to design the candidate are independently derived and attacked. A failure either repairs the common basis or remains a named contradiction; it cannot be patched with a container-specific privileged entry. |

## 5. Structural performance rows

| ID | Requirement | Evidence required for PASS |
|---|---|---|
| P-1 | Same representation | Each protected route can use the same payload layout, required metadata, allocation count, indirection count, ownership traffic, and asymptotic algorithm as a best known implementation of the same contract. |
| P-2 | No universal tax | Prefix/full owners pay no sparse tag, bitmap, generation, reference count, atomic, topology tag, or dynamic-dispatch tax. Sparse, shared, recyclable, and concurrent contracts pay only state required by their own semantics or chosen algorithm. |
| P-3 | Erasure | Proof terms, roots, permissions, protocol state, and abstraction evidence have no runtime representation unless the contract or selected runtime validation needs the corresponding state. |
| P-4 | Check accounting | Dynamic checks remain when required by the public contract or selected validation route. Elision requires machine proof and produces exact fact provenance; no proof mode weakens source semantics. |
| P-5 | Honest claim strength | Structural parity proves absence of a forced language tax, not measured throughput parity. Any throughput, code-size, compile-time, proof-production, or writer-floor claim requires a separately authorized preregistered measurement. |

## 6. Research completion rule

The D14 research may close only when every G, B, D, and P row has one of the
following exact dispositions:

1. `PASS`, with a constructive derivation or exact authoritative evidence;
2. `IRREDUCIBLE_GATE`, with a fully scoped semantic entry under G-1 through
   G-7; or
3. `OWNER_NON_GOAL`, with an existing owner ruling.

`PENDING`, `plausible`, `future family`, `BASIS?`, analogy to Rust, and absence
of a discovered counterexample are not completion dispositions. Independent
hostile review must reconstruct the row partition, attack at least one witness
per retained authority, and authenticate the exact report bytes. Repository
durability, both verification gates, English-only contents, and a clean commit
remain mandatory.
