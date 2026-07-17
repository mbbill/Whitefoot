# Candidate C Sparse Repair Candidates

Date: 2026-07-15

Status: bounded paper comparison complete; mandatory Sparse Repair Gate stop.
No proposal in this file modifies Candidate C v0 or authorizes implementation.

Sparse Repair Gate disposition: `SPARSE-SELECT: SR-PROFILE`.

## 1. Result in one sentence

Retain `SR-PROFILE`, a closed profile-indexed sparse automaton, as the next
Candidate C research hypothesis because it gives finite paper routes to all
five frozen operations without an identified structural tax, avoids copying a
full operation catalog into every sparse representation, and remains distinct
from B and A by denying ordinary libraries authority to define profiles,
relations, transitions, facts, or cleanup programs.

This is a paper selection only. It is not formal safety, exact derivability,
implementation feasibility, generated-code parity, or measured performance.

## 2. Shared exact C0 leaf proposals

These leaves are dependencies shared by `SR-CLOSED` and `SR-PROFILE`. They are
not sparse-family operations and cannot grant payload liveness.

### GROUP-MATCH-1 — checked portable control-group operation

Inputs are a checked control region, an offset whose `W`-byte footprint is
inside that region including any representation-owned replicated tail, one
closed control classifier ID, and one listed query: `match_tag(tag)`,
`match_empty`, or `match_vacant`. `W` is selected from a closed target-profile
table. The result is a `W`-bit mask whose bit `i` is set exactly when the
classifier accepts byte `offset+i` for the query.

Each target row lists a scalar implementation and optional SIMD lowering with
identical masks for every byte vector. Feature selection changes code choice,
not semantics. The operation reads control bytes only, performs no payload
access, creates no occupancy fact by itself, writes no memory, and has no
failure or partial progress. Unsupported target/width pairs select the listed
scalar row; they do not call an untyped helper or accept a writer opcode.

The sparse automaton may consume the mask together with its own root/version/
slot relation to produce candidate or probe facts. The mask alone grants no
payload place or key match.

### ROOT-ALLOC-1 — checked vacant physical-root acquisition

Inputs are a checked nonzero byte size, a listed power-of-two alignment, and a
closed shortage policy (`return_failure` or `abort`). Success returns one fresh
physical root covering exactly the allocator-returned extent and records the
actual extent if it differs from the request. Recoverable failure is
precommit, returns no root, and changes no owner. Abort produces no subsequent
program state. Release consumes one empty root exactly once with the same row.

The row creates raw vacant bytes only. It initializes no payload, creates no
typed place liveness, copies no owner, and knows no sparse profile. Resize is
not a primitive row: the sparse protocol acquires a new root, stages or moves
payload through its own rule, commits root transfer, and releases the empty old
root. Custom allocator behavior remains outside this frozen slice.

## 3. Common semantic notation

The paper candidates use the following notation only for comparison:

- `Stable(profile, root, version)` means ordinary sparse operations may run.
- `Occupied(root, version, slot, place)` is an affine access token produced only
  when the selected closed relation classifies the control slot as live.
- `Vacant(root, version, slot, old_class)` is an exact-use entry token.
- `Phase(kind, ...)` is exclusive and non-escapable. While it exists, no Stable
  operation or pre-phase access token can be used.
- A logical commit may contain several physical writes. Exclusivity prevents an
  intermediate write from producing source-visible authority.
- A version is a static/checker epoch unless the selected representation already
  stores a runtime generation. The proposal adds no runtime version field.

Tokens and phases describe checker authority. They are not required runtime
objects, tags, guards, allocations, or branches.

## 4. SR-CLOSED — operation-closed sparse profiles

### Definition

Each compiler-known sparse profile contains a complete operation catalog. For
the audited representation, `ByteTagSparse-v1` fixes:

- the FULL(hash-fragment), EMPTY, and DELETED byte classes;
- payload/control layout and replicated-tail bounds;
- group and probe behavior;
- lookup, find-vacant, insert, replace, replace-with, remove, resize, and
  in-place-rehash state machines;
- every entry/result source map;
- every fact producer and invalidator; and
- every normal, failure, abort, partial-progress, and abandonment disposition.

An ordinary library may instantiate and wrap the profile but cannot add an
operation or reinterpret a state. A new control encoding or transition is a new
compiler-owned profile or operation and therefore a language change.

### Five-operation route

- Lookup calls the profile's group/probe procedure, produces `Occupied` only
  for FULL, and stops only on its EMPTY rule.
- Vacant insertion consumes the profile-specific vacant entry, performs its
  hidden logical commit, and publishes the result borrow at the new version.
- Replacement selects one closed owner-disposition branch, including duplicate
  key retention and callback restore/remove.
- Removal uses the profile's local group rule to select EMPTY or DELETED and
  consumes every prior slot token.
- Rehash enters the profile-specific exclusive phase. Resize stages bytes as
  non-owners until liveness transfers at root commit. In-place rehash gives
  transition-DELETED a live meaning only inside its phase and fixes partial
  cleanup for every remaining pending-live owner.

All five matrix rows are `CLOSED` and show no structural delta from the frozen
reference account.

### Implementation and writer shape

The checker dispatches on profile plus operation ID and validates one dedicated
state machine. The compiler/backend receives canonical lowering per operation.
Diagnostics can name the exact operation and phase. A writer chooses a low-level
representation profile and ordinary named-container code wraps it; no project
name is privileged.

This is locally simple but globally repetitive. Every new admitted sparse
representation duplicates lookup, entry, replacement, deletion, rehash,
cleanup, access, and fact definitions even where their structure is identical.

### Pros

- Most concrete authority and lowering boundary.
- Smallest reasoning scope for one operation.
- Strong diagnostic locality.
- Clearly remains Candidate C.

### Cons

- Largest semantic, compiler, backend, and review surface per profile.
- Cross-product growth across profiles, operations, result forms, facts, and
  target rows.
- Rehash and partial cleanup are too large to duplicate safely without strong
  evidence that representation differences require it.
- Most likely alternative to evolve into many container-adjacent special cases.

### Falsifier

An independently required sparse representation that differs only in control
classification but nevertheless requires a copied full operation catalog would
demonstrate avoidable family-surface growth.

## 5. SR-PROFILE — closed profile-indexed sparse automaton

### Definition

One compiler-owned `SPARSE-AUTOMATON-1` schema contains only closed fields:

1. layout kind and checked control/payload slot mapping;
2. finite control classes `live(tag)`, `vacant_stop`, and `vacant_continue`;
3. optional transition-local reinterpretations keyed by a closed phase ID;
4. group classifier and probe-step ID;
5. counter equations for each transition;
6. erase classifier ID;
7. a finite access-token source map;
8. a finite fact-schema set; and
9. admitted transition-template IDs.

The only transition templates in this repair are `INSERT-1`, `REPLACE-1`,
`REMOVE-1`, `RELOCATE-1`, and `REHASH-1`. Their state and outcome enums are
fixed. Profile fields select rows from compiler-known tables; they cannot hold
source functions, predicates, proof terms, bytecode, callbacks, cleanup code,
paths, symbols, or container identity.

Ordinary libraries may instantiate an admitted profile ID and supply ordinary
hash/equality behavior through C0-2. They cannot define a profile or compose a
new transition. Adding a schema field, profile row, phase, transition, access
map, or fact kind is a reviewed Candidate C change.

### Exact control-to-payload rule

In `Stable`, the automaton checks the profile's control classifier for a slot.
Only `live(tag)` produces `Occupied(root, version, slot, payload(slot))`.
`vacant_stop` and `vacant_continue` produce no payload authority. Group masks
only select slots to classify; they do not bypass this per-slot rule.

Inside `REHASH-1`, the exclusive phase table may classify the same physical
DELETED byte as `pending_live`. That authority requires the unique phase token,
cannot escape, and disappears when the phase advances or closes. Stable
operations always interpret DELETED as vacant.

### Transition rules

`INSERT-1` consumes `Vacant` and the offered owners. Before logical commit,
failure returns the unchanged table and offered owners. Commit makes exactly
one payload place live, updates control/counters, invalidates the old version,
and issues the result map. Physical metadata-before-payload or payload-before-
metadata writes remain unobservable under the exclusive phase.

`REPLACE-1` has a closed owner-disposition table: keep stored key and replace
value; take one owner and restore a replacement; or take one owner and remove
the slot. Each branch states stored, offered, displaced, returned, destroyed,
and restored owners. Occupancy facts survive only through a freshly issued
postcommit fact; all old content and borrow facts invalidate.

`REMOVE-1` consumes `Occupied`, moves the owner once to the result, selects
`vacant_stop` or `vacant_continue` through the profile's finite erase classifier,
updates counters, and issues new probe facts. There is no recoverable edge after
the owner take.

`RELOCATE-1` acquires a vacant root through ROOT-ALLOC-1. Each source-live payload
is read once into destination bytes that remain non-owner staged images. Hashing
continues to borrow the live source. On failure, staged bytes are discarded and
source owners remain. On commit, all source liveness transfers to corresponding
destinations, old access invalidates, and the empty old root is released. This
matches the one-copy reference structure without two live owners.

`REHASH-1` first changes stable FULL slots into phase-local pending-live slots.
Each step either restores FULL without movement, relocates one pending owner to
a vacant-stop slot, or swaps it with another pending owner and carries the
displaced owner in the phase token. Successful closure leaves only stable
classes. Exceptional partial cleanup destroys every still-pending owner once,
turns those slots vacant, preserves processed FULL owners, invalidates all phase
facts, and returns only the documented survivor state.

### Access and facts

The fixed access maps are `VacantEntry`, `OccupiedEntry`, `Bucket`,
`ReturnedBorrow`, and `OwnedResult`. Each contains exact root, version, slot,
place, source owner, and invalidating transition classes. Temporary entry or
callback objects create no root.

The fixed sparse fact kinds are `ControlClass`, `GroupCandidates`, `Occupied`,
`Vacant`, `ProbeContinue`, `ProbeStop`, `KeyHit`, `GrowthBudget`,
`PendingLive`, `RelocationMap`, and `FinalState`. C-11/C0-5 lists each producer,
consumer, and invalidator. Facts erase unless their producer is an existing
runtime comparison; no new metadata field is introduced.

### Implementation and writer shape

The checker interprets one finite automaton and validates profile rows at
toolchain construction time. Compiler lowering specializes profile and
transition IDs, so dynamic profile dispatch is not required. Backend lowering
uses GROUP-MATCH-1 and ROOT-ALLOC-1 rows. Diagnostics name profile, phase,
transition, owner, root/version, and failed invariant.

Writers see a small fixed family vocabulary and ordinary typed entry/results;
they do not write relations, transitions, proofs, or cleanup graphs. Compared
with SR-CLOSED, AI generation chooses among fewer semantic operations and the
checker can diagnose the shared phase model consistently.

### Pros

- Closes all six known definition gaps with one bounded semantic package and
  two separate machine leaves.
- Reuses insertion, replacement, removal, relocation, and rehash rules without
  making their composition open to ordinary source.
- Preserves representation-selected metadata and zero unrelated-shape cost.
- Specializes statically and needs no project-name recognition.
- Smaller fact and diagnostic surface than a full per-profile catalog.

### Cons

- More complex central checker logic than SR-CLOSED.
- Profile-schema design becomes a sensitive language boundary.
- The compiler-owned profile/transition cross-product can still grow and needs
  explicit accounting.
- Only one independent sparse project has been calibrated, so generality beyond
  the frozen representation is unproved.
- Staged-byte relocation and partial-progress cleanup still require formal and
  adversarial validation before implementation.

### Falsifier

The hypothesis fails if another required efficient sparse mechanism needs a
control relation, phase, transition, cleanup, or provenance form outside the
closed schema, and adding it either creates project-specific recognition,
unbounded table growth, a structural tax, or writer-defined proof authority.

## 6. SR-ORTHOGONAL — open factoring control

### Definition

This alternative exposes generic storage/control relations, place access,
exact-use transition nodes, commit graphs, cleanup graphs, provenance maps, and
fact producers. Ordinary libraries compose them into lookup, insertion,
replacement, removal, and rehash protocols.

It can describe the reference structure on paper. The problem is authority:
Hashbrown rehash needs a writer-selected phase relation, loop invariant,
relocation map, displaced-owner invariant, and partial-cleanup set. Validating
arbitrary compositions requires Candidate B's public orthogonal algebra. If the
library supplies general propositions or proof terms for those invariants, it
converges further toward Candidate A.

All five matrix rows are therefore `CONVERGES-B`. This is not a statement that
B is undesirable; B remains the later compression challenge. It means this
alternative is not a Candidate C repair under the frozen role separation.

### Pros

- Small primitive vocabulary.
- Greatest ordinary-library representation freedom.
- Potentially compresses common transition and provenance concepts.

### Cons

- Open composition, graph validation, termination, equivalence, and diagnostics
  become the main checker problem.
- Cleanup and rehash invariants are no longer a finite Candidate C table.
- A general predicate escape turns it into A.
- Weak-writer and AI behavior depends on a much larger construction space.

### Falsifier

If the component grammar can be closed tightly enough to cover all five rows,
reject malformed graphs locally, and avoid writer-defined invariants without
recreating `SR-PROFILE`, then the `CONVERGES-B` classification is too strong.

## 7. Uniform comparison

| Dimension | SR-CLOSED | SR-PROFILE | SR-ORTHOGONAL |
|---|---|---|---|
| Five-operation routes | 5 `CLOSED` | 5 `CLOSED` | 5 `CONVERGES-B` |
| Semantic surface | Full catalog per profile | One schema, five templates, finite profile rows | Small nodes, open graphs |
| Representation freedom | Only complete admitted profiles | Admitted rows within closed schema | Library-composed relations |
| Transition reuse | Low | High within the sparse family | High across families |
| Provenance precision | Dedicated per operation | Fixed access maps parameterized by profile | Depends on composed maps |
| Cleanup precision | Dedicated and local | Fixed phase/outcome tables | Requires graph/invariant validation |
| Fact-schema count | Repeated per profile | One fixed sparse set | Open producer composition |
| Checker state | Many small dispatchers | One larger finite automaton | General graph/proof checker |
| Compiler/backend | Canonical per operation | Static specialization of profile/template IDs | Must lower arbitrary compositions |
| Diagnostic locality | Strongest | Strong phase/invariant vocabulary | Weakest until graph diagnostics mature |
| Writer-visible complexity | Fixed profile APIs | Fixed profile and entry APIs | Protocol construction vocabulary |
| AI stability risk | Many similar special operations | Smaller closed choice set | Largest construction space |
| Structural no-tax result | No delta identified | No delta identified | No delta identified, but authority open |
| New-family pressure | High profile duplication | Lower while schema holds | Replaced by open composition growth |
| Cross-family growth | Operation/profile cross-product | Profile/template table cross-product | Unbounded component graph space |
| Candidate convergence | Stays C | Stays C under closed admission | Converges to B, potentially A |

## 8. Hostile paper review

1. **Project-name laundering:** rejected. SR-PROFILE rows contain representation
   and transition IDs only; no library path, symbol, API, or key/hash identity.
2. **Writer predicate laundering:** rejected by schema. No field accepts a
   predicate, proof term, bytecode, callback, or cleanup program.
3. **Control/payload desynchronization:** logical commit requires exclusive
   Phase authority and publishes facts only after all physical writes.
4. **Normal DELETED used as live:** rejected. Pending-live interpretation
   requires the non-escapable REHASH-1 phase token.
5. **Stale entry reuse:** rejected. Every mutation changes the checker version
   and consumes old entry, bucket, borrow, and probe tokens.
6. **Hash/equality creates authority:** rejected. Calls receive only an already
   authorized borrow and can add KeyHit, never Occupied.
7. **Resize duplicates affine owners:** rejected at paper level. Destination
   bytes are staged non-owners until one bulk liveness-transfer commit.
8. **Resize failure drops copies:** rejected. Staged non-owner bytes are
   discarded; source owners remain live.
9. **In-place exceptional double drop:** rejected by the phase partition:
   processed FULL survives, pending-live drops once, and carried displaced
   owner is accounted in the phase token.
10. **Fact resurrection after same-address reuse:** rejected. Versions, not
    addresses, identify facts.
11. **SIMD overread:** rejected by GROUP-MATCH-1's checked W-byte footprint and
    representation-owned replicated tail.
12. **SIMD/scalar semantic skew:** rejected by the exact mask equivalence
    requirement; not yet mechanically tested.
13. **Allocation creates liveness:** rejected. ROOT-ALLOC-1 returns vacant raw
    bytes only.
14. **Dense/fixed tax:** none identified. Sparse profile state and code exist
    only for a selected sparse instantiation.
15. **Hidden runtime version/tag:** rejected. Version and phase are checker
    authority unless reference algorithm state already materializes them.
16. **Profile-table explosion:** remains an open falsifier. The proposed repair
    reduces duplication but one project cannot establish ecosystem bounds.
17. **Silent convergence to B:** rejected for the selected hypothesis by closed
    compiler ownership of schema, profiles, templates, and facts. Opening any of
    them to library composition changes the classification.
18. **Silent convergence to A:** rejected because no general proposition or
    writer proof enters admission.

The review finds no paper-level invariant violation in SR-PROFILE, but it is not
an adversarial safety proof. The unresolved proof, implementation, code-shape,
measurement, and independent-demand questions remain explicit.

## 9. Structural cost conclusion

For SR-PROFILE, every runtime item is already charged to the reference sparse
representation or algorithm: control bytes and replicated tail, counters,
probe state, operation-local progress, displaced owner, resize root, and
callback state. Tokens, versions, phases, source maps, and facts are static
checker objects unless their corresponding reference state already exists.

The proposal identifies no forced initialization, zeroing, payload copy beyond
the reference relocation, extra allocation, metadata field, indirection,
branch/check, scan, atomic/fence, machine event, code-size requirement, or
asymptotic change. This is structural paper accounting only. Compiler lowering
could still fail to erase static structure or match reference code shape.

## 10. Sparse Repair Gate

`SR-CLOSED` closes the frozen slice but has avoidable per-profile catalog growth.
`SR-ORTHOGONAL` is useful as B's later compression direction but is not a C
repair. `SR-PROFILE` is `CLOSED` for all five rows, violates no identified
invariant, adds no identified structural event, and stays within a finite
compiler-known Candidate C family boundary.

The exact gate result is `SPARSE-SELECT: SR-PROFILE`, as a further-research
hypothesis only. Candidate C v0 is unchanged. The next logical work would be a
separately authorized formal/adversarial validation of the automaton and a
second independent sparse-demand audit before any language or implementation
decision. Neither is authorized here.

Work stops at this gate. Stage 2 and every implementation or further audit
remain unauthorized.
