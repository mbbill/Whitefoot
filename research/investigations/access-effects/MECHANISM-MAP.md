# Requirements and mechanisms: a map for the ownership redesign

Research date: 2026-09-16. This is a language-design investigation, not an
amendment to the active specification (v0.57) and not a claim of compiler
support. It belongs to the access-effects investigation beside
[RESEARCH.md](RESEARCH.md) (literature and hypothesis), [DESIGN.md](DESIGN.md)
(two candidate systems) and [CASES.md](CASES.md) (incremental cases). It is the
frame in which those candidates are evaluated; it selects nothing. Supersede it
in place when the analysis moves on.

## 1. Purpose

A Rust-style reference is one value that answers several unrelated questions
at once: may this be dereferenced now (memory safety), may anyone else write
while it is held (interference), what may the optimizer assume (aliasing),
which proved facts survive (frame), and who releases the storage (lifecycle).
Whitefoot inherited that coupling and tightened it. The coupling is why a change
to ownership drags regions, effects and parallel permission along, and why such
a change is hard to think through to completion.

This document states the requirements independently, records the least
information each one needs, maps every current mechanism to the requirements
it serves, lists the mechanism families known to serve each requirement, and
records the couplings a candidate design must respect.

## 2. Requirements, stated independently

Each requirement gives (a) what must hold, (b) the least a checker must know,
(c) where the check happens, (d) what is not its job. The constitution's
meta-constraints apply to every mechanism and are listed once at the end.

### R1 Sequential memory safety

- (a) No read of storage holding no value; no access to storage that has ended
  (freed, reallocated, released at scope exit, vacated by a relocating move);
  no access under a stale layout.
- (b) For the storage an access names: live, initialized in the selected part,
  layout current. These are properties of storage, not of the pointer used to
  reach it.
- (c) At each access, against the state of the named storage at that point.
- (d) Not its job: preventing two pointers to one storage; preventing
  sequential aliased writes; deciding parallel overlap.

### R2 Resource lifecycle accounting

- (a) Every allocation and external resource is released exactly once; linear
  obligations are discharged; where bounded memory is promised, the bound is
  provable.
- (b) Which binding holds the release obligation of which storage, and whether
  it is definitely discharged on every path.
- (c) At consuming operations, at every scope exit and at every join. Release
  decisions are static, never runtime flags (LIV-1).
- (d) Not its job: who may read or write the storage.

### R3 Value classes and transfer

- (a) Copy values duplicate freely and carry no obligation; affine values move
  at most once and may take a compiler-derived release; linear values are
  consumed exactly once. A move transfers a value with its obligations; a take
  leaves a hole; a put fills one; a replace exchanges.
- (b) Each value's class; for each transfer, its source and destination
  storage.
- (c) At the transfer.
- (d) Not its job: pointer validity. A value leaving storage `'a` is an event
  on `'a`'s state, a hole or an end, which R1 consumes.

### R4 Aliasing facts for the optimizer

- (a) The backend receives facts it may use without guards: two accesses never
  overlap; a storage is not written during a span; a loaded value is invariant
  over a span.
- (b) Distinctness of storages, by identity, field, proved index or proved
  range; write footprints over spans.
- (c) At lowering, from retained checked facts; never re-derived by analysis.
- (d) Not its job: safety. A missing fact costs speed, never correctness.

### R5 Data-race freedom and parallel permission

- (a) Two statements or two iterations may overlap only when their footprints
  do not interfere; a data race is unrepresentable.
- (b) Read and write footprints of each statement over storages; distinctness
  between footprints; data dependencies.
- (c) PAR-1 windows and PAR-2 loops today; a future thread construct at spawn
  and join.
- (d) Not its job: sequential safety. Permission never changes acceptance
  (design/language/parallelism.md).

### R6 Frame: proof-fact retention

- (a) After a write or a call, exactly the facts whose support may have changed
  are dropped; every other fact survives.
- (b) The written footprint of each operation, keyed as facts are keyed.
- (c) ENT-5 kill events at commits and call boundaries.
- (d) Not its job: safety or permission; it decides proof precision.

### R7 Modular, signature-only checking

- (a) A call is judged by the callee's signature alone (FN-1). A signature can
  say what the callee needs of each storage on entry and leaves on exit: a
  hole, an ended storage, a replaced backing, a returned pointer into an
  argument, a fresh allocation.
- (b) Storage identities nameable in signatures; entry and exit states per
  identity; effect footprints per identity.
- (c) At the call for the caller; once at the returns for the callee body.
- (d) Not its job: inferring anything from bodies.

### R8 Storage placement and relocation

- (a) A value lives somewhere: frame, heap store, arena, inline in a container.
  Some operations relocate storage, such as a relocating move or a reallocating
  push, and interior pointers name the storage before relocation.
- (b) Which operations end or replace which storage; whether aggregates travel
  as bytes or as handles.
- (c) At the relocating operation, as a state event on the source storage (R1)
  and a contract clause (R7).
- (d) Not its job: who may access; only when a storage stops existing.

### R9 Shared mutation among several holders

- (a) Several long-lived pointers to one storage, each writing occasionally.
  Sequentially this is safe; across threads a write needs synchronization.
- (b) Sequentially: nothing beyond R1 and R6. Concurrently: which
  synchronization primitive holds the storage's state facts while no thread
  holds them.
- (c) Sequentially at each access; concurrently at acquire and release.
- (d) Not its job: forbidding aliasing to strengthen R4. That trade belongs to
  R4 and is local to the storages whose identities coincide.

### R10 External resources as ordinary objects

- (a) Files, sockets, mappings and devices obey the same state, effect and
  proof rules as memory (constitution).
- (b) One identity per external resource; its states; which operations
  transition them; what an external actor may change between operations.
- (c) At each operation, through ordinary contracts.
- (d) Not its job: host mechanisms or scheduling (effects.md rejects mechanism
  categories).

### Meta-constraints

- M1 Checking is deterministic and budget-free: fixed terminating families plus
  explicit certificates; no solver, timeout or work budget selects acceptance.
- M2 No runtime safety check or trap in accepted programs; a condition that can
  be false is a typed outcome with real control flow.
- M3 No writer-accessible unsafe escape.
- M4 The writer is an AI: verbosity is cheap; irregularity, special cases and
  non-local diagnostics are expensive; the specification stays small.
- M5 The accepted shapes are the fast shapes; a failed parallel permission
  leaves a program sequential rather than rejected.

## 3. The hazard ladder

The ladder adds one feature at a time and asks what can go wrong and the least
a checker must know to refuse it. It fixes the minimal information for R1, R2,
R3 and R8 and shows where R4, R5 and R6 first appear. Notation: `ptr<'a, T>` is
a copyable pointer whose type names storage `'a`; `state('a)` is a checker fact
in `{Init, Uninit, Gone}`; owners are ordinary affine bindings. This notation
is illustrative and is not a surface-syntax proposal.

### Step 0: scalars, pointers, sequential execution

```text
(p, own_a) = alloc(10)
q = p
write(p, 1); x = read(q)     // legal, 1: sequential aliased access needs no judgment
v = take(p); read(q)         // reject: read of a hole.      needs state('a)
dispose own_a; read(q)       // reject: use after end.       needs state('a) = Gone
dispose own_a                // reject: double release.      needs an affine owner
```

Precision rather than safety: a write through any pointer to `'a` kills facts
about `'a`'s contents. ENT-5 already kills by support; the support becomes the
identity. No read/write conflict judgment appears at this step.

### Step 1: struct fields

```text
s: S in storage 'a; px = ptr_of(s.x)      // px: ptr<'a.x>
write(s.y, 1); read(px)                   // legal: 'a.x unchanged
v = take(s.x); read(px)                   // reject: 'a.x = Uninit; put restores it
t = move s; read(px)                      // reject: bytes moved to 'b, 'a = Gone, never restored

b = heap_box(...); pc = ptr_of(deref(b).x)
b2 = move b; read(pc)                     // legal: the box moved, its content storage did not
```

The distinction that matters is whether an operation relocates storage, not
whether the value is a struct or a container. A take is a value leaving
storage; a relocating move is the storage ending.

### Step 2: fixed array

```text
pi = ptr_of(a[i]); pj = ptr_of(a[j])
v = take(a[i])                            // state('a[i]) = Uninit
read(pj)                                  // requires state('a[j]) = Init: needs j != i, else reject
```

Index disjointness first appears here, for sequential state precision, before
any parallelism. Without holes it is not needed.

### Step 3: growable vector

```text
v: Vector in 'v; fact backing('v) = 'b
pe = ptr_of(v[i])                         // pe: ptr<'b[i]>
push(v, 5)                                // contract: len0 < cap0 => backing unchanged;
                                          //           else 'b = Gone, backing('v) = fresh
read(pe)                                  // requires state('b) != Gone: provable iff len0 < cap0
```

The container case is Step 0 plus one contract clause: push may end storage,
as free does. Rust invalidates every borrow unconditionally; here the proof of
`len0 < cap0` decides.

### Step 4: calls

Contracts state entry and exit states per identity and which storages end or
are replaced. No loans are needed.

### Step 5: parallel overlap

The read/write conflict table appears here and only here: two overlapping
statements may not write a storage the other reads or writes.

### What each mechanism is for, by the ladder

| Mechanism | Hazard or goal it serves | First needed |
|---|---|---|
| Per-storage state (`Init`, `Uninit`, `Gone`) | hole read, use after end | Step 0 |
| Affine owner binding | double release, leak | Step 0 |
| Storage identity in the pointer type | state stays precise under aliasing; optimizer distinctness | Step 0 |
| Kill facts by identity on write | proof precision, not safety | Step 0 |
| Contract-visible storage-ending operations | relocation, reallocation, free | Steps 1 and 3 |
| Index and range disjointness proofs | state precision with holes in arrays | Step 2 |
| Read and write footprints | parallel permission only | Step 5 |
| Borrow modes, regions, exclusivity | no row above needs them | none |

The last row is the ladder's finding: borrowing is not the mechanism that
refuses any hazard once storage state and identity exist. Its remaining role
is the interference and optimizer facts of R4 and R5, which footprints over
identities can carry, and the default clean shape of M5, which section 7
records as an open coupling.

## 4. Today's mechanisms mapped to the requirements

P marks the requirement a rule primarily serves, S a secondary one. The
bucket column says which of the ladder's roles the mechanism plays:
**perm** exists only to keep permission-carrying references sound, **life**
is lifecycle accounting, **intf** is interference or optimizer facts,
**frame** is fact retention, **class** is value-class semantics, **proof**
is the deterministic proof discipline. Rule text is the authority; the
mapping is this document's reading of it.

| Mechanism | Rules | R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 | R9 | R10 | Bucket |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Copy or affine classification, explicit `move`, dead-root kill | OWN-1 | S | P | P | | | | | S | | | class |
| Borrow modes `&`, `&uniq` | OWN-2 | S | | | P | P | P | S | | P | | perm |
| Lexical regions, outlives, incomparable caller regions | OWN-3 | P | | | | | | S | | | | perm |
| Named-region liveness, region-checked storing and returning | OWN-4 | P | | | S | S | | S | | | | perm |
| Resolved-place exclusivity and suspension | OWN-5 | | | | P | P | P | | | P | | intf via perm |
| View origin sets; no slice-valued join | OWN-5, VIEW-1, VIEW-2 | S | | | P | P | | P | | | | perm |
| Holder resolution; statement-scoped and candidate-position child reborrow | OWN-6 | | | | S | S | | P | | | | perm |
| Overlap of resolved places; proved range disjointness | OWN-7 | S | | | P | P | P | | | | | intf |
| Reject when unsure | OWN-8 | | | | | | | | | | | proof |
| Optimizer noalias consequence | OWN-9 | | | | P | | | | | | | intf |
| Borrow-storage duration | OWN-10 | P | | | | | | | S | | | perm |
| Loop body as a region; per-iteration liveness agreement | OWN-11 | P | S | | | S | | | | | | perm |
| Call substitution; argument loans; effect projection | OWN-12 | | | | S | P | S | P | | | | intf |
| Match ownership; arm-scoped binder reborrows | OWN-13 | S | S | P | | | | | | | | class, perm |
| Returned reborrow; other reborrow forms deferred | OWN-14 | | | | | | | P | | | | perm |
| Join-checked liveness; unconditional scope-exit release | LIV-1 | S | P | | | | | | | | | life |
| One `set` commit; read-out; no observable hole | LIV-2 | P | S | S | | | | | | | | life |
| Store identity is a region; brand resolution | PROV-1 | | P | | | | | S | P | | | life |
| Linearity by capability presence; `dispose`; release graph | PROV-6 | | P | S | | | | | S | | | life |
| Borrow-free storage | STOR-5 | P | | | | | | | P | | | perm |
| Effect rows: three categories, formal-rooted static paths | EFF-1 | | | | P | P | P | P | | | P | intf |
| Exhibited-row exactness; call-boundary projection; reclamation contribution | EFF-2 | | | | S | P | P | P | | | | intf, frame |
| `pure` licensing | EFF-3 | | | | P | | | | | | | intf |
| No writer-visible capability category | CAP-1 | | | | | P | | | | | | intf |
| Window permission with loans and footprints | PAR-1 | | | | | P | | | | | | intf |
| Counted-loop permission: accumulator, affine element maps, adjacent ranges | PAR-2 | | | | | P | | | | | | intf |
| Support kill at commits and calls | ENT-5 | | | | | | P | | | | | frame |
| Routed relations at calls | CALL-6 | | | | | | S | P | | | | proof |
| Invariants and explicit certificates | ENT, section 15 | S | | | | S | | | | | | proof |

The full 44-row inventory with verbatim rule quotes, dependency lists and a
bucket per row is part A of
[EVIDENCE-mechanism-survey-2026-09-16.md](EVIDENCE-mechanism-survey-2026-09-16.md).
Its count: 19 rows exist only to keep permission-carrying references sound,
6 are lifecycle accounting, 6 interference, 4 fact maintenance, 4 value
class, 5 other.

Two readings of the table:

- Every **perm** row serves R1 only because the reference is also the
  permission: regions, named-region liveness, borrow-storage duration,
  loop-body regions and borrow-free storage exist to stop a permission-bearing
  reference from outliving its storage. Under per-storage state, the storage
  ending is the fact and no reference needs a duration.
- Every row that serves R4 and R5 does so through exclusivity and place
  overlap. The information those consumers actually read, and whether
  identity footprints carry it, is section 5's question.


## 5. Mechanism families per requirement

For each requirement, the families known to serve it, the information each
needs, and its fit with M1 to M5. Representative works are named for
orientation; the survey with constraint scoring and citation status is in
[EVIDENCE-mechanism-survey-2026-09-16.md](EVIDENCE-mechanism-survey-2026-09-16.md)
part C, and RESEARCH.md keeps the primary-source table. Fit phrases are this
document's assessment.

### R1 Sequential memory safety

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Permission-carrying references with regions or inferred lifetimes | Rust NLL, Oxide; WF today | reference durations, exclusivity | flow analysis over regions or NLL | M1 needs a fixed lifetime solution, which is why WF chose lexical regions; M4 poor: non-local diagnostics; M5 good | stored references, holes and split parts are inexpressible |
| Storage identity in the pointer type plus per-identity state | Alias Types 2000, L3 2005, Verus `PointsTo` 2023, DESIGN.md Candidate A | which storage each pointer names; state per storage; storage-ending events | type equality plus state facts at each access | M1: equality and finite states are fixed families; M2, M3 met; M4: local diagnostics | proof burden where identity is data-dependent: indices, containers, recursive structures |
| Stateful views over a linear context | ATS 2005, Cogent 2016 | view assertions per location, including uninitialized | linear type rules | M1 to M3 met; M4: a second small language | recursive structure needs dependent views |
| Typestate and path-sensitive property automata | Strom and Yemini 1986, Bierhoff and Aldrich 2007, ESP 2002 | finite state per tracked object; permission per reference | finite-state dataflow | M1 met; M5: safety facts only | precision collapses under aliasing without a permission system underneath |
| Region-based memory management | Tofte and Talpin 1997, Cyclone 2002 | region per allocation; outlives | region typing | M1 met; M5 poor: coarse lifetimes leak or copy | no per-object free inside a live region |
| Runtime-validated references | Vale generational references, Mezzo adoption | generation per allocation | runtime compare and trap | M2 violated | disqualified; a baseline only |

### R2 Resource lifecycle accounting

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Affine owners with compiler-derived release, static at joins | WF OWN-1, LIV-1; Cogent; Austral | owner per storage; liveness agreement at joins | linear typing over structured control flow | all met; no drop flags | join disagreements are restructured by the writer |
| Linear obligations with explicit consumption where no capability is present | WF PROV-6; Wadler 1990; Linear Haskell 2018 | provider capability in scope | linear typing | all met | explicit routes only |
| Capability-indexed deallocation | Capability Calculus 2000 | linear capability per region, consumed by free | linear typing | all met | region granularity; per-object free needs L3-style locations |
| Runtime drop flags or ownership indicators | Rust MIR, MLIR ownership-based deallocation | none static | runtime flag | M2 violated; WF refused it in LIV-1 | a hidden bit the source could have stated |
| Reference counting with reuse | Perceus 2021 | inferred ownership | RC insertion | M5 violated for ordinary data | its reuse analysis is a useful precedent only |

### R3 Value classes and transfer

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Copy, affine, linear classes with explicit `move` | WF today; Austral; Linear Haskell multiplicities | class per type | typing | all met; one spelling per meaning already | none |
| Uniqueness typing for in-place update | Clean 1996, Futhark 2017 | uniqueness per reference | attribute propagation | met; Futhark shows M5 gains for arrays | uniqueness is not an obligation; R2 needs its own rule |
| Explicit take and put on inline storage | Cogent 2016, ATS views | which slot holds a value, which is a hole | linear typing with slot state in the type | met; this is WF's hole-and-refill requirement with no runtime flag | slot state in types grows with nesting |
| Handle passing for aggregates, storage fixed for life | why-whitefoot section 10 pools; Hylo `inout` conventions | storage identity stable across calls | representation choice plus contracts | met; enables interior pointers across calls | contracts must say whether the callee ends the storage |
| Ghost permission separated from the runtime value | Verus `Tracked<PointsTo>`, L3 | pointer plus a separate linear permission | linear checking of the ghost term, erased | met for the linear part | permissions threaded by hand through every call |

### R4 Aliasing facts for the optimizer

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Exclusive references lowered to `noalias` | Rust `&mut`; WF OWN-9; Stacked and Tree Borrows as the model | live exclusive loans | borrow checking | met; parameter granularity only; loaded pointers unmarked | the gap why-whitefoot section 5 measures |
| Per-location store typing | Alias Types, L3, Capability Calculus | a name per storage; distinct names are disjoint | syntax-directed | met; the strongest per-storage facts; per-span `alias.scope` possible | location polymorphism and existential packing at structure boundaries |
| Region and effect annotations on code | DPJ 2009, Lucassen and Gifford 1988, Regent | region partition; effect summary per call | effect subsumption and region disjointness | met if the index fragment is fixed | array partitioning needs arithmetic disjointness, where solvers usually enter |
| Uniqueness and purity | Clean, Futhark, Cogent | uniqueness; purity | syntax-directed | met for arrays | weak for pointer graphs |
| Proof-derived separation exported to the backend | RefinedRust 2024, Verus, Iris | separation assertions | tactics or SMT | M1 violated by the derivation; usable only with fixed fact forms and explicit steps | non-local failures |
| Compiler-side recovery | LLVM alias analysis, loop versioning | none | analysis under guards | contradicts M5's premise | guards and code bloat |

### R5 Data-race freedom and parallel permission

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Loans plus effect footprints over resolved places | WF PAR-1, PAR-2 | loans, rows, place overlap, affine index images, range proofs | fixed judgment per window or loop | met today | loans restate what rows and identities say |
| Footprints over storage identities with distinctness facts | DPJ 2009, Regent's static part, the CSL parallel rule (O'Hearn 2007) | identity footprints; identity distinctness; index or range proofs | the same fixed judgment with identities as roots | met if identity parameters are distinct by default or proved | a proof where two footprints come from one container |
| Fractional or counting permissions | Boyland 2003, Bornat et al. 2005, Chalice, Viper | permission amount per location | entailment; SMT as deployed | M1 needs a fixed split discipline; the counting form is deterministic | verbose bookkeeping |
| Sendability by capability | Pony 2015, Rust `Send`/`Sync` | capability per type | typing | met; thread-level only | a capability lattice to teach |
| Runtime dependency tracking | Legion runtime | none static | runtime | M2 and M5 violated for permission | scheduler cost |
| Purity and uniqueness | Futhark, Cogent | purity; uniqueness | syntax-directed | met for data-parallel shapes | no general statement overlap over mutable structures |

### R6 Frame: proof-fact retention

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Kill by written footprint keyed as facts are keyed | WF ENT-5; Dafny `modifies` as notation; Low* `modifies loc h0 h1` | write footprint per commit and call | deterministic kill | met; the key becomes the identity | precision follows identity granularity |
| Ownership-derived framing | Prusti 2019, Flux 2023, Creusot 2022 | the ownership structure already present | framing from types; only value facts to a solver | the framing half is deterministic; WF keeps that half and replaces the value half with explicit steps | shared state falls back to explicit permissions |
| Store typing as the frame | Alias Types, L3 | full store type before and after each operation | syntax-directed rewriting | met | unaffected memory is still named; needs abstraction for large modules |
| Separation-logic frame rule | Reynolds 2002, O'Hearn | footprint as a separating conjunct | structural when separation is syntactic | met when separation is by distinct names; search otherwise | inductive predicates need explicit fold and unfold |
| Implicit dynamic frames and region logic | Smans et al. 2009, Viper; Banerjee et al. 2008 | accessibility predicates or region expressions | verifier | M1 violated as deployed | solver |

### R7 Modular, signature-only checking

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Store-type pre and post in the signature | Alias Types, Capability Calculus | store-in and store-out, including holes and vanished locations | syntax-directed subsumption at the call | met; the signature literally says which storage becomes a hole or ends | signatures grow; location polymorphism and existentials are mandatory |
| Requires and ensures over identities and states | Vault 2001, Mezzo `consumes`, Verus, DESIGN.md Candidate A | identity parameters; entry and exit states | signature check at call and return; routed states reuse CALL-6 | met | every callee that changes storage state must say so |
| Modifies and reads clauses as notation | Dafny, Low* | footprint sets, `fresh`, `old` | SMT frame axioms as deployed | the notation fits M4; the discharge violates M1 and must be replaced by a footprint algebra | none if the discharge is syntactic |
| Lifetime-parameterized signatures returning interior pointers | Rust elision, WF FN-1 provenance, OWN-14 | which parameter a result borrows from | region inference or provenance rules | met, but expresses durations only, never holes or frees | a rule per case |
| Prophecies or backward functions for mutable borrows | RustHorn 2020, Creusot, Aeneas 2022 | final value of a borrow | solver or translation | M1 poor; unnecessary when access checks state per operation | complexity |
| Existential identities for fresh results | Alias Types `exists`, DESIGN.md `exists P` | the pack | pack and unpack | met | an unpack step at the caller |

### R8 Storage placement and relocation

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Storage classes with explicit placement rows | WF STOR-*, BLK-2 | placement per value | typing | met today | none |
| Relocation as a name change | L3 `realloc: cap(ρ) -> exists ρ'. cap(ρ')`; this document Steps 1 and 3 | old and new storage identity; the transfer between them | type rule plus contract clause | met; the most explainable form; makes push conditional on `len < cap` | every derived pointer must be re-derived after a relocation |
| Take and put on inline storage | Cogent, ATS | which inline slots are full or empty | linear typing | met; inline-in-container without a hidden tag | slot state multiplies with nesting |
| Projections for interior access | Hylo subscripts, Swift accessors | the projected path and its span | static exclusivity over paths | M2 partially: Swift falls back to dynamic checks for class storage | interior pointers may not escape the projection |
| Pinning | Rust `Pin` | a pinned flag | typing | expresses less than a state event | idiom burden |
| Handles instead of interior pointers | why-whitefoot section 10 | index plus bounds proof | bounds proofs | met; already taught | one indirection |

### R9 Shared mutation among several holders

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| Sequential: state per storage, aliasing allowed | Candidate A; L3 unrestricted pointers | state per identity | R1 and R6 | met; nothing further needed | facts weaken where identities coincide |
| Identity and permission separated by a brand | GhostCell 2021 | a brand shared by cells; one token whose borrow grants access to all of them | ordinary type check on the token | met at zero runtime cost | brand-coarse: no concurrency inside one brand |
| Lock owns the storage's state while unlocked | CSL resource invariants, Chalice monitors, Verus locks | lock invariant naming identities | acquire yields facts, release requires them | met when threads arrive; cost only at acquire | lock cost at the write, none for the hold |
| Ghost protocols over shared state | Iris STS, fictional separation (Jensen and Birkedal 2012), CAP 2010, Verus state-machine sharding | per-role transitions | ghost proofs | M1 needs a fixed protocol checker | proof burden; a second sub-language |
| Manifest sharing on channels | Balzer and Pfenning 2017 | acquire and release types | session typing | acquire is a typed outcome with real control flow, which matches M2 | interaction with R4 across an acquire is not worked out |
| Fine-grained access permissions | Bierhoff and Aldrich 2007 | permission per reference | typestate checking | expressible as footprints plus a read-only flag | five modes to teach |
| Interior mutability with runtime flags | Rust `RefCell`, `Cell` | none static | runtime flag | M2 violated; why-whitefoot rejects the hole | every fact conditional |

### R10 External resources as ordinary objects

| Family | Representative | Information needed | Checked by | Fit | Cost |
|---|---|---|---|---|---|
| External state as identities with declared states and opaque contents | Low* external state, VST external state, Iris external resources | identity per resource; state machine; opacity of contents | contracts | met; no special rule per resource | contracts per operation |
| Capability tokens for I/O | Effekt, Scala capture checking | capability in scope | typing | expressible as an identity in scope | a second vocabulary |
| Effect categories for host mechanisms | early WF `external`, `blocks` | mechanism labels | rows | rejected by effects.md: mechanisms, not state | none |

### The three decouplings the literature already made

Every design above that scores well on M1 to M5 makes one of three cuts,
and the cuts are what the hazard ladder found independently:

| Cut | Where it is made | What it buys | What it costs |
|---|---|---|---|
| Pointer from capability | Alias Types, L3, Verus `PointsTo` | copying a pointer stops implying R1 or R4 consequences; R7 signatures can say "ends `'a`, produces `'b`" for reallocation | the capability is threaded or looked up; recursive structures need packing |
| Identity from permission | GhostCell | R9 at zero runtime cost; R4 survives because the permission is still exclusive | permission granularity is the brand's |
| Effects from ownership | DPJ | R4 and R5 are checked from effects on code rather than ownership of values; the object model stays conventional | effect annotations restate information the type system nearly has |

The decomposition in section 6 makes all three cuts at once: the pointer
carries identity only, permission is the storage's state and the operation's
footprint, and interference is judged from effects over identities.

Two further findings from the survey bear on M1:

- In Prusti, Flux, Viper and implicit dynamic frames, the framing and
  permission half is decidable accounting while the value half goes to a
  solver. That split is where WF's automatic and explicit boundary belongs:
  framing is derived, value facts are explicit `use` steps.
- The `modifies` and `reads` clauses of Dafny and Low* are the best notation
  in the literature for "this call frees, reallocates or returns an interior
  pointer"; only their solver discharge is unusable. The clause form can be
  kept with a syntactic footprint discharge.

### What the consumers of ownership facts read today

Part B of the evidence file reads the specification and the compiler to find
what each consumer of ownership information actually consumes, and whether
identity footprints carry the same information. Condensed:

| Consumer | Reads today | Under identity footprints | Verdict |
|---|---|---|---|
| PAR-1 window footprints | effect rows projected onto resolved places; OWN-7 overlap; consumed `own` actuals; arena regions as a separate access kind | the same over identities; the arena region stays its own identity kind | equal |
| PAR-1 argument loans | the borrow mode of each actual, independent of the callee's row: a `pure` callee taking `&uniq` still holds an exclusive loan | no counterpart: a footprint is an access, a loan is an exclusion | the one difference, below |
| PAR-1 argument-expression reads | per-operand reads; address formation is not a read | forming `ptr<'a, T>` is manifestly not an access of `'a` | equal or cleaner |
| PAR-2 accumulator, affine element maps, adjacent-range partitions | one binding and a closed operator set; retained `ProvedAffineIndexMap` and `ProvedRangePartition` from discharged OP-4 proofs | the same proofs hung on identities; `RangeId` is already an identity in all but name | equal |
| PAR-2 exclusive-loan containment | every exclusive loan must be iteration-own or inside a partition | same difference as the loan row; but the model regains the shared-versus-uniq distinction that the checked tree erases today, which forces PAR-2 to refuse every non-view borrow of enclosing storage | difference plus a gain |
| ENT-5 fact kill | OWN-7 overlap of a projected write against fact support; an element-versus-descriptor bit; a per-parameter reach table from declared mode and type | identity footprints state element, descriptor and range reach directly | equal or better |
| EFF-2 call projection | static struct paths; a view actual projects through its origin set, which the compiler implements as at most one origin and fails closed on callee-returned views | substitution is ordinary type instantiation; a returned view carries its identity | strictly better |
| Backend storage reuse | a three-valued source mode plus IR liveness; its one assumption is that a result may alias a consumed input | identity equality states the sameness directly | equal or better |
| Backend alias and effect metadata | **nothing**: the current emitter states it "emits no overflow or alias promises", and no `noalias`, `alias.scope`, `readonly` or `memory` attribute is emitted anywhere; the why-whitefoot measurements are retired-prototype evidence | per-identity scopes on loaded pointers, per-tile scopes for PAR-2, `initializes` from a `writes`-only row, cross-procedural `noalias` on declarations | a channel to build, under either model |

Two consequences for this map:

- **Today the whole borrow apparatus buys exactly one backend effect: PAR-1
  and PAR-2 overlap actualization.** R4 is a requirement no current mechanism
  serves; the noalias story in why-whitefoot section 5 was measured on the
  retired prototype, which re-derived per-field scopes from resolved places at
  emission. A storage identity in the type is the first-class form of that
  re-derivation.
- **The one behavioral difference is the loan channel.** Today a `&uniq`
  actual denies overlap against any access of the same place even when the
  callee only reads; identity footprints would grant it. Execution is unaffected,
  since two reads do not race, but PAR-1's recorded ground is source
  equivalence, "an implementation may not construct a loan state the source
  checker refuses", and permission-judgment.md lists "treating exclusive borrows
  as writes" among its rejected alternatives. Under the decomposition there is
  no exclusive loan to preserve; whether a call may still claim exclusivity
  beyond its row is a decision to record, not a derivation.

Two soundness conditions the decomposition must state explicitly:

- Two distinct identity parameters may not be assumed distinct unless the call
  site discharged their distinctness, including sub-identities of one storage
  such as `'v[i]` and `'v[j]`. OWN-7 today takes the opposite default: formal
  origins "never establish that two actual sources are disjoint". The
  default-distinct choice in section 7 is sound only with that call-site
  obligation.
- A pointer derived from another pointer, today's reborrow, shares its
  parent's identity or names a sub-identity of it; it never mints a fresh one.
  OWN-9 forbids exactly the noalias pair a fresh identity would assert.

### Threads, locks and external resources under the decomposition

Part D of the evidence file studies what the decomposition needs when threads,
locks and external resources arrive. Its findings, condensed:

| Need | Mechanism | New vocabulary? | Runtime cost | Status |
|---|---|---|---|---|
| Lexically scoped fork and join | PAR-1 already is the CSL parallel rule with syntactic footprints; the window end moves to the join statement | none | fork and join edges | CONCURRENCY-CATALOG section 12 already maps `thread::scope` onto PAR-1 |
| Parallel width not written in source | a PAR-2 iteration-exclusive affine range loan `[c*i+b, c*(i+1)+b)`, proved by the arithmetic PAR-2 already runs | none | none | the catalog's own most recurring cost; a PAR-2 refinement, not a thread construct |
| A child that outlives its block | a linear task handle carrying the child's exit contract over the transferred identities; join consumes it | one linear type | a wait | Chalice `fork`/`join` tokens, Verus join handles; an unjoined child's obligations are an open question |
| Conditional states crossing a join | collapse `ite(c, s1, s2)` at any boundary whose condition the other side cannot name; publish the condition as a returned value when it is needed | none | none | the one place the identity model asks for something CSL does not; sound, untested for precision |
| A lock over identities no caller owns | an ordinary nominal declaring a custody set and an invariant; acquire adds the custody facts at the invariant and a linear guard; release checks the invariant and erases every fact about the custody identities | one nominal kind | one mutex per critical section | CSL resource invariants, Chalice monitors; the invariant's precision condition is unchecked against WF's fact model |
| Two lock-bearing calls actually overlapping | a second permission level: race freedom plus invariant preservation without PAR-1's source-order equality | yes, an interference category CAP-1 excludes | the mutex | a decision, not a derivation; DPJ and Regent decline it to keep determinism |
| Several holders, occasional writes, across threads, paying only at the write | protocol-governed shared regions with a stability judgment and a memory model | a second model | one atomic per write | recommended against; the phase and epoch pattern the catalog already measures beats a read-write lock on the read path at zero reader cost |
| Externally writable storage: shared mappings, device registers, DMA | a **foreign** identity qualifier: no contents fact is retained across any operation; extent facts stay ordinary | one qualifier | none | passes the constitution only if spelled over interference, "another agent may write it without an edge in this program's order", never over origin |
| File descriptors, sockets, anonymous mappings | nothing: a descriptor is three identities, the wrapper with the close obligation, the open-file description with the cursor, and the contents any agent may write | none | none | RESEARCH.md's I/O witness already separates these subjects |

The lock finding is the sharp one. The custody sketch works under the
decomposition, but two `alloc` calls on one lock both exhibit `writes(lock)`,
so PAR-1 denies their overlap: safe and useless. Letting them overlap needs a
permission whose guarantee is weaker than PAR-1's source-order equality. That
is a language decision the map records; it selects nothing.

## 6. A decomposition that keeps the requirements separate

The ladder and the table suggest one decomposition. It is [DESIGN.md](DESIGN.md)'s
Candidate A stated requirement by requirement, with two refinements from
[CASES.md](CASES.md): storage state at a join is a term over the captured
branch condition rather than a lattice value, and signatures carry entry and
exit states. Nothing here is selected.

| Requirement | Mechanism in the decomposition | Status relative to today |
|---|---|---|
| R1 | Pointer types name a storage identity; the checker holds `state(identity)` facts; every access checks the named storage's state; storage-ending operations are contract-visible | new: identity on pointers, per-identity state |
| R2 | Affine owner bindings, `dispose`, capability-by-provider linearity, release graph, static release decisions at joins | unchanged: OWN-1, LIV-1, LIV-2, PROV-6 |
| R3 | Copy, affine, linear classes; `move`, take, put, replace as state events on storage; `own` aggregates travel as handles so storage stays put; relocation only by explicit `move` into other storage | changed: handle passing; relocation set explicit |
| R4 | Distinctness from identities: fresh allocations distinct, distinct identity parameters distinct by default, fields and proved ranges distinct; write footprints over spans give per-load alias scopes | changed: facts from identity, not exclusivity; no alias fact is emitted today under either model (section 5) |
| R5 | PAR-1 and PAR-2 over footprints of identities; the loan clause retires; index and range proofs unchanged | changed: roots of footprints |
| R6 | ENT-5 kills by identity footprint; call projection substitutes identity arguments | changed: key of the kill |
| R7 | Identity parameters in signatures and nominals (today's `region_params` spelling, meaning identity only); `requires`/`ensures` over states; existential identities for fresh results; states routed by result variant as CALL-6 routes relations | new: state clauses |
| R8 | Storage classes unchanged; interior pointers name the backing identity; reallocation is a contract-declared replacement with a conditional exit state | new: backing identity |
| R9 | Sequential: nothing; concurrent: a lock holds an identity's state facts while unlocked (section 5, R9) | new when threads arrive |
| R10 | External resources are identities with declared states and an opaque-contents rule | new: external identities |

What this retires, what it keeps, what it adds:

| Retired | Kept | Added |
|---|---|---|
| OWN-2 modes as permission (a read-only pointer becomes an interface flag, not a loan) | OWN-1, OWN-8, LIV-1, LIV-2, PROV-6, ENT-5, CALL-6, certificates | identity parameters on pointer types |
| OWN-3 lexical regions and outlives; OWN-4 named-region liveness; OWN-10 borrow-storage duration | OWN-7 as footprint disjointness over identities | per-identity state facts, conditional at joins |
| OWN-5 exclusivity, suspension, view origin sets, no slice-valued join | EFF-1/2 exactness and projection, rooted at identities | contract clauses for entry and exit states, ended and replaced storage |
| OWN-6, OWN-13 binder borrows, OWN-14 reborrow family | PAR-1, PAR-2 structure and proofs | default-distinct identity parameters with declared aliasing |
| OWN-11 loop body as region; STOR-5 borrow-free storage | PROV-1 spelling `'s` with identity meaning only | handle passing for `own` aggregates; explicit relocation set |
| PAR-1 loan clause | | existential identities for fresh results |

## 7. Couplings and change impact

These are the places where one choice reaches several requirements. A
candidate that changes a row's choice must re-check every requirement the row
names.

| Choice | Reaches | Why |
|---|---|---|
| Identity granularity: per allocation, per field, per element, per proved range | R1 precision with holes, R4, R5, R6, M4 spec size | one refinement vocabulary serves state, distinctness, footprints and kills |
| Distinct identity parameters distinct by default, aliasing declared | R4, R5, M5 floor, M4 call-site proof burden, soundness | keeps Rust's clean-shape default without a mode on pointers; sound only if every call site discharges distinctness for every pair, sub-identities of one storage included, which inverts OWN-7's current default; costs a proof where two arguments come from one container |
| Sequential aliased writes allowed | R4 where identities coincide, R6 kill precision, M5 | safety stays with state; facts weaken only for storages the writer let coincide |
| `own` aggregates passed as handles | R8, R7 contracts must say whether the callee ends the storage, backend calling convention | identity survives the call only if storage does not move |
| Storage state as a term over captured conditions | M1 cost in condition atoms, R1 and R2 definiteness at scope exit and loop heads | replaces path-sensitive analysis with fact discharge; definiteness rule replaces drop flags |
| The set of storage-ending operations | R1 completeness, R7 contract vocabulary | every member must be contract-visible: free, dispose, scope exit, reallocation, relocating move |
| Effect roots become identities | EFF-1 grammar, R5, R6, R7, generics: nominals carry identity parameters | struct-held pointers can be named in rows only through the struct's identity parameters |
| Backing identity for containers | R7 push and reserve contracts, R8, interior pointer types | an existential backing that a contract may replace |
| Read-only interface on pointers | R4 readonly facts, API design | a type flag, never a loan or a duration |
| Locks holding identity state | R5 threads, R9, M2 | synchronization cost lands on the write, not on the hold |
| Per-storage state as the primary mechanism rather than an add-on | R1, R2, R3, M1 | affine-replacement.md rejected a bare take because a hole open across statements "needs per-place vacancy flow, prohibition or repair of every scope-leaving edge in the window, and a meaning for an exclusive borrow over a vacant referent"; the decomposition pays the first two deliberately, as state facts and the definiteness rule, and the third disappears because pointers carry no permission |
| Modes retired instead of refined | R4, R9, M4 | LEX-1 defers a two-axis mode vocabulary "exclusivity x write-permission, adding frozen/exclusive-read and an explicitly bounded shared-write form"; identity plus footprints answers the same questions without a second axis on pointers |

## 8. Open questions before a candidate can be selected

1. Is the storage-ending set complete and contract-visible? Scope exit,
   `own` parameter passing, container insertion, enum payload moves and
   returns are the cases to check one by one.
2. What identity does an element or a backing carry: a path identity
   `'v[i]`, an existential backing unpacked at pointer formation, or both?
   Interior pointers into containers depend on the answer.
3. Distinct-by-default identity parameters: what the writer declares to allow
   aliasing, and what the call site must prove.
4. Effect rows over identities: grammar, exactness in both directions, and
   whether rows stay writer-written or become derived from contracts.
5. State terms: atom growth, definiteness at cut points, and whether one
   `ensures` clause can state a reallocating push.
6. Whether reads need anything at all in sequential code, or only writes
   and lifecycle events; the ladder says only writes and events.
7. Threads, locks, atomics and external identities: the minimal additions
   (section 5, R9 and R10).
8. A soundness plan: an exhaustive small-model check like
   [access-state](../../experiments/access-state/RESULTS.md) first, a formal
   model later.
9. Migration: `&` and `&uniq` as sugar over pointer plus state clause; region
   syntax retirement; pattern cards.
10. Conditional states at a fork or join: the collapse rule for `ite` states whose
    condition the other side cannot name is sound but untested for precision.
11. A lock over identities no caller owns fits the decomposition, but two calls
    on one lock deny each other under PAR-1; letting them overlap is a second
    permission level without source-order equality, which CAP-1 excludes today.
    This is a decision to record, not to derive.
12. Externally writable storage needs one qualifier, foreign, spelled over
    interference rather than origin; whether an internal object shared with
    another thread carries the same qualifier decides its constitutionality.
13. A spawned child that is never joined holds obligations its parent cannot
    discharge; CAP-1 says nothing about obligation leaks across a thread
    boundary.
14. The loan channel: whether a call may claim exclusivity on an identity beyond
    its row, as a `&uniq` actual does today, or whether the row is the complete
    interference contract. Execution is safe either way; PAR-1's recorded
    source-equivalence ground is what changes.
15. R4 has no current mechanism: the emitter states no alias promises, so the
    alias-fact channel must be built under either model; the identity model
    makes the prototype's per-field re-derivation a first-class fact.

## 9. Relationship to the other files

[RESEARCH.md](RESEARCH.md) records the literature and the hypothesis;
[DESIGN.md](DESIGN.md) records Candidate A and Candidate B in full;
[CASES.md](CASES.md) records the incremental cases under Candidate A. This map
adds the requirement-level frame and the hazard ladder that justifies the
minimal information. CASES.md case B08 is refined here into the
state-as-term rule of section 6; nothing else in those files changes.
