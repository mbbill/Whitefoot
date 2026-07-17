# Minimal Privileged Capability Basis: Hostile Review

Status: PASS for limited research archival, 2026-07-15. This verdict validates
the report as an honest negative synthesis and gate-direction recommendation.
It does not select the public storage basis, prove whole-systems derivability
or no-tax performance, authorize Stages A or B, change G0 or H-STORE, or
authorize a language, specification, compiler, library, experiment, or
production action.

## 1. Exact reviewed inputs

Two independent reviewers passed the same final bytes with no remaining P0 or
P1 finding:

| Artifact | SHA-256 |
|---|---|
| `MINIMAL-PRIVILEGED-BASIS-REPORT.md` | `3f1998623e2278585867f512a20da2d9c218b1b0a114f26b28932a655254acf8` |
| `SEQUENTIAL-WITNESS-DERIVATIONS.md` | `fd334c57d76f064d69d2bd9c6f114c94ff6caad6fbd666c76bf98947de2d3deb` |
| `SYSTEMS-DOMAIN-ADMISSION-MAP.md` | `ebab8f2e0592d9524d0fd7db7382d961b60b52bab5e9e519a797b19606cdf4bd` |

## 2. Attacks and dispositions

### 2.1 Gate authentication and nonforgeability

Initial drafts treated digests and review identity as authority. That did not
authenticate owner approval, especially for `TRUSTED_FOREIGN`, and left
rollback, replay, cache, binary, and helper substitution underspecified.

Disposition: the final report defines one parenthesized compiler acceptance
predicate. Base entries must occur in the embedded root. A capsule must have a
canonical encoding, exact authority-tuple membership in the active signed
approval snapshot, a valid signature under the installation-pinned approval
key, matching target/frame/program/policy/disposition scopes, and either a
valid `CERTIFIED` proof or the exact signed `TRUSTED_FOREIGN` frame assumption.
The protected snapshot epoch, toolchain identity, artifact revalidation, and
rollback/replay behavior are explicit. Stage A must still freeze and attack the
full design before production selection.

### 2.2 Ordinary proof versus privilege

An earlier draft allowed the same word, certificate, to describe both an
ordinary proof-bearing library and a privileged capability capsule. That could
have moved unforeseen containers behind the gate and defeated ordinary-library
generativity.

Disposition: a proof-bearing artifact is ordinary when it uses only existing
public checked semantics and introduces no primitive, helper, alternative
lowering, backend behavior, opaque representation rule, foreign assumption,
effect/fact kind, proof rule, or TCB edge. Every producer may submit it to the
fixed verifier without owner approval. A capsule is required exactly for a new
machine, runtime-helper, backend, ABI, OS, device, or foreign semantic edge,
including an alternative compiler lowering.

### 2.3 Safety authority versus abstract correctness

The first synthesis overgeneralized from storage safety to proof of arbitrary
container semantics. It treated universal shadow state as the only dynamic
alternative and used H-STORE key/position coherence to motivate a general
proof assistant.

Disposition: the final trilemma applies only to propositions consumed for
memory access, ownership, destruction, release, unique borrowing, or optimizer
facts. Such authority requires static proof, runtime validation/enforcement, or
explicit ledgered trust; ordinary source cannot mint trust. Sortedness, key
uniqueness, graph reachability, and other abstract semantics remain ordinary
correctness unless explicitly promoted to a checked contract or fact.

H-STORE now checks `k < U` before reading `position[k]`, then uses `p < n` plus
dense-prefix liveness to authorize key and value reads. Key equality determines
the abstract lookup result. Removal takes both key and value slots and restores
exact dead tails. Its frozen `ST-SPARSE` requirement is therefore recorded for
re-adjudication as a possible mechanism overconstraint rather than used to
preselect a general proof logic.

### 2.4 Bounded storage calculus

The first seven-role sketch hid arbitrary set predicates, silently selected
recoverable allocation, omitted full/inline adoption and existential sealing,
used imprecise mutation access, and claimed generic swap and replacement through
intermediate states outside its bounded grammar.

Disposition: the final candidate is explicitly sequential, unique-owner, and
bounded to `Empty`, `Full`, `Prefix`, `Interval`, and `Pair` live shapes. It
follows current trap allocation and leaves OD-1 open. Mutation requires exact
exclusive slot authority with incompatible borrows absent. Full/inline
adoption, opaque generative sealing, focus, and cleanup remain named blockers.
Nine schematic roles include atomic shape-preserving `swap` and `replace`.
Relocation is derived only when its intermediate and successor states remain in
the grammar; arbitrary relocation stays blocked.

### 2.5 Cleanup

The initial certificate language smuggled in executable cleanup. A proof cannot
run an `O(n)` destruction loop, and current STOR-3 has no ordinary user
destructor.

Disposition: the report records an explicit fork between exact-linear manual
close and a restricted verified disposer. The latter is operationally a
verified user destructor and requires termination, recursion, effect,
allocation, trap, callback, and nested-resource rules. No cleanup model is
selected.

### 2.6 Performance and staging

Earlier drafts claimed a constructive route for every systems domain and a
structural zero-tax result. They also mixed sequential, sparse, recursive,
shared, and relaxed-atomic obligations into one unclosable lock and required
measurement while forbidding construction and execution.

Disposition: the 26-domain file is an admission/dependency map, not a
derivability proof. The performance conclusion is only that no universal
runtime metadata tax has been shown necessary. Stage A freezes gate
authentication. Stage B freezes bounded sequential static semantics and a
future performance protocol without executing it. Stage C opens only after a
formal counterexample or later separately authorized measurement establishes
need for general predicates. Later semantic families remain separate. Exact
IR, code size, runtime, checker, certificate, and protected-path evidence is
still required.

## 3. Final verdict

The following limited conclusions pass:

1. one authenticated sealed capability admission verifier is a credible gate
   direction for owner review;
2. one gate does not imply one semantic authority or one trusted assumption;
3. ordinary proof over existing semantics is not privilege;
4. the nine-role bounded storage model is a useful next static-design
   candidate, not a selected public basis; and
5. complete ordinary-library systems derivability and no-tax performance
   remain open.

No candidate construction, implementation, execution, language decision, or
production action follows from this PASS.
