# Minimal Privileged Capability Basis

Status: hostile-review-corrected research synthesis for owner review,
2026-07-15. This report selects one research-level admission-gate direction,
records why no complete public capability basis can yet be selected, narrows
one bounded sequential unique-owner candidate, and maps the remaining systems
envelope to independently counted semantic authorities admitted through the
gate. It does not claim whole-systems derivability or zero cost, select source
syntax, change the language or specification, implement a compiler feature,
construct a standard library, restart E0.1, or authorize a production
mechanism.

## 1. Result

The research had been operating one layer too high. G0-Core correctly built a
finite demand ledger, and the dense lock correctly exposed hard ownership,
failure, cleanup, and performance obligations. However, choosing among five
dense-container mechanisms before choosing the common privileged foundation
would optimize a local mechanism before fixing the language's real extension
boundary.

The corrected result has four layers:

1. **One gate:** a sealed capability admission verifier with a fixed
   compiler-embedded root and out-of-band extension capsules. It admits
   compiler-shipped capabilities and separately authorized third-party frame
   instances through the same closed verification and audit path. Ordinary source
   has no keyword, attribute, path, module name, command-line flag, or package
   identity that can grant this authority.
2. **One selected safety-authority rule, not one selected logic:** every
   proposition consumed for safety or optimization must come from static
   proof, runtime validation/enforcement, or explicit trusted authority.
   Bounded deterministic reasoning is tested first; a general proof-bearing
   fallback is adopted only if a required efficient shape proves it necessary.
3. **One bounded sequential storage candidate:** nine transition roles are a
   useful API decomposition for sequential unique-owner storage, but they also
   require generative/existential sealing, full or inline storage adoption,
   exact normal-exit rules, and a selected cleanup model. The current language
   provides none of those as a complete package.
4. **One preserved objective:** sequences, deques, hash tables, trees, arenas,
   graphs, strings, and other policies should remain ordinary generic xlang
   code, so the toolchain need not ship named containers. The present research
   does not yet prove that objective for every family.

The selected direction is called the **sealed capability admission verifier**
in this report. The schematic storage candidate is called the **checked
resource-transition algebra**. These are research labels, not proposed source
spellings. The gate is selected only at the architectural level. The storage
candidate failed universal-derivability review and is retained only as a
bounded next design object. Neither is a production design.

This is a stronger design than a private `unsafe` standard library. Access
control answers who may introduce primitive authority. It does not prove that
the privileged implementation is correct, minimize the semantic authority
behind the gate, or let an ordinary library derive an unforeseen structure.
The selected admission-gate direction requires all three properties; the
public generativity proof remains open.

The existing G0 and dense artifacts remain useful, but their role changes:

- G0 is the finite completeness and cost test for the basis.
- The dense lock is a high-resolution adversarial test suite for the basis.
- The five dense candidates are no longer the next design choice.
- OD-0 through OD-5 are not the next owner action.
- No existing frozen artifact is rewritten from hindsight.

## 2. The distinction that prevents a false minimum

"One backdoor" can mean five different things. Only the first is plausibly one.

| Count | Meaning | Selected result |
|---|---|---|
| Gate count | How privileged declarations enter the toolchain | Exactly one sealed capability admission verifier direction |
| Ordinary-source admission spelling count | How ordinary source requests privilege | Zero |
| Proof model count | How ownership, state, and effects are justified | Exact logic unselected: bounded deterministic tier first; counted proof-bearing fallback only if required |
| Semantic authority count | Distinct machine or runtime actions | `N`, never inferred from the gate count and independently ledgered |
| Proof-rule count | Rules trusted or certified by the verifier | `M`, explicitly counted by the later static design |
| Unproved TCB assumption count | Compiler, helper, backend, frame, and bootstrap assumptions | `K`, explicitly counted and never renamed as proof |
| Public storage-role count | Safe composable transition families in the sequential sketch | Nine schematic roles; role count does not bound proof-language size |

Combining allocation, atomics, volatile I/O, a system call, and a typed slot
transition behind one opaque opcode would not make them one semantic
operation. They have different effects, failure behavior, memory-order rules,
optimizer consequences, and backend implementations. A single declaration
family is valuable; a single untyped magic operation is not.

This distinction also answers the `box` question. A `box` keyword can be one
writer-visible operation, but it owns one complete value. It does not establish
spare capacity, partial initialization, sparse occupancy, exact live-subset
destruction, relocation, or metadata/payload coherence. Adding hidden meanings
to `box` until it supports all of those cases would create a container-shaped
universal primitive with an unbounded semantic surface.

## 3. Selected gate: a sealed capability admission verifier

### 3.1 Shape

The trusted compiler installation embeds a canonical fixed root manifest: a
closed capability/effect/fact schema, reference-policy digest, certificate
verifier identity, generic ABI mechanism, and base capability root. It
constructs compiler-internal identities for accepted entries. The user parser
cannot parse privileged declarations, and no accepted ordinary artifact form
can introduce an internal capability capsule. Current PROG-1 has no module
loader; any future module or binary-artifact design must preserve this rule.
There is no analogue of a user-selectable `-parse-stdlib` flag. File location,
text name, signature, symbol, layout,
package name, module name, build script, environment variable, cache entry, or
source attribute confers no authority.

The same gate accepts two origins:

1. base entries shipped in the fixed root; and
2. third-party capability capsules submitted out of band with a
   machine-checkable certificate or an exact trusted-foreign disposition, plus
   the existing AI-authored/human-approved gate record.

Both origins are decided by one compiler acceptance predicate. A base entry is
accepted only when its canonical bytes and identity occur in the embedded root.
A capsule entry is accepted only when all of the following hold:

```text
capsule schema and canonical encoding are valid
and its full authority tuple is present in the active approval snapshot
and that snapshot has a valid signature under the installation's pinned
    approval-root public key
and target, frame, program, policy-root version, and disposition scopes match
and (
  disposition is CERTIFIED and proof validation succeeds
  or
  disposition is TRUSTED_FOREIGN and the active signed snapshot contains the
      exact frame/TCB assumption disposition
)
```

The approval snapshot is an installation-protected, monotonically versioned,
canonical registry maintained outside all source, dependency, build-script,
environment, and package inputs. Its signed digest and epoch are part of the
compiler/conformance identity. A receipt is not authority by itself; only
membership in the active authenticated snapshot is. Replaying a receipt under
another target, program, frame, disposition, or policy-root version fails.
Replaying the same approved entry under the same pinned snapshot is harmless
and deterministic. A lower snapshot epoch is rejected by the protected
installation state; a deliberately installed older snapshot has a distinct
toolchain identity and cannot masquerade as the current one.

A capsule may instantiate only the fixed root's schema, proof rules, effect
kinds, fact kinds, ABI actions, and reference semantics. It cannot install a
new verifier, proof rule, effect kind, fact channel, backend plugin, memory
model rule, or arbitrary lowering. A capability requiring any such extension
requires a new fixed root and therefore a distinct reviewed toolchain identity.

This second route is necessary for a general systems language with no shipped
standard library. A project may need a new C ABI, device register map, OS API,
allocator, or target operation that the compiler release did not anticipate.
Ordinary source and dependencies cannot create, select, activate, or smuggle a
capsule or approval snapshot. The out-of-band approval operation updates the
authenticated protected registry. Ordinary builds may refer only to accepted
public checked signatures by exact identity.

Each capability entry has an authority tuple rather than a reusable name:

```text
(logical_id,
 contract_digest,
 representation_digest,
 target_lowering_or_helper_digest,
 certificate_digest,
 frame_assumption_digest,
 disposition,
 target_scope,
 frame_scope,
 program_scope,
 policy_root_version,
 review_digest)
```

The logical ID, review digest, or certificate digest alone carries no
authority. Consuming artifacts and caches pin the exact used-entry tuple,
approval-snapshot identity, or dependency-cone digest, so an unrelated package
update need not invalidate everything and a changed contract cannot reuse an
old proof. Each entry records at least:

- public checked signature, if any;
- opaque runtime representation or layout rule;
- backend lowering or runtime entry identity;
- resource and ownership pre-state and post-state;
- normal, failure, and abort outcomes;
- cleanup or release obligation;
- borrow and result-provenance relation;
- optimizer facts produced, transferred, and invalidated;
- proof or explicitly counted frame/TCB assumption identity;
- target availability and backend-parity obligations; and
- dependency-cone and review identities.

Every capsule entry has one of two dispositions:

- `CERTIFIED`: the helper or lowering has a certificate against the fixed
  reference semantics; or
- `TRUSTED_FOREIGN`: an opaque OS, vendor, or hardware implementation is an
  explicit program-specific frame/TCB assumption. It exports only the
  conservative checked result, ownership, and fact attenuation recorded in the
  capsule and is never described as machine-proved.

Most new C or OS APIs are instances of the fixed generic ABI action and can use
a capsule without a compiler release. A genuinely new machine effect or fact
kind cannot.

An entry may itself expose a public checked callable signature. A bundled
wrapper is ordinary code only when it is accepted as ordinary code over those
public checked entries. If a wrapper body directly uses a private primitive or
cannot be checked through the public basis, that wrapper is another certified
entry and is counted in the same gate and proof/TCB ledger. There is no hidden
third category of "ordinary but privately privileged" wrapper.

The ordinary-proof boundary is syntactic and semantic, not discretionary. A
proof-bearing module remains ordinary when its executable body uses only
already public checked language and capability semantics, and its certificate
only derives propositions in the fixed proof/fact schema. It may introduce no
primitive, helper, alternative lowering, backend behavior, opaque
representation rule, foreign assumption, effect kind, fact kind, proof rule,
or TCB edge. Every producer may submit such a module to the same fixed verifier
without owner approval.

A capsule is required exactly when an artifact binds a new machine, runtime
helper, backend, ABI, device, OS, or foreign semantic edge, including an
alternative compiler lowering of an existing operation. `CERTIFIED` means that
edge is proved against the fixed reference semantics; it does not turn an
ordinary proof-bearing library into a privileged capsule. This boundary
prevents the gate from absorbing unforeseen containers and defeating ordinary-
library generativity.

The root and capsule can have a readable authoring representation for review without
making that representation xlang source syntax. Its canonicalizer/parser,
schema, and output bytes have pinned identities and reject duplicate, unknown,
ambiguous, or noncanonical encodings. This internal authoring language counts
against TCB, review, and META-5 cost even though it grants no ordinary-source
spelling.

### 3.2 Threat boundary and artifact rules

"Nonforgeable" in this report means **nonforgeable by accepted ordinary source
and dependency artifacts relative to a trusted conforming compiler
installation**. Replacing the compiler or its verifier is outside the language
guarantee and remains an explicit TCB attack.

A conforming installation has no source, environment, search-path, package,
cache, plugin, build-script, or command-line override for its reference-policy
and verifier roots. A development compiler with another root has a distinct
compiler and conformance identity and cannot emit a conforming artifact
identity.

Privileged compiler identities never serialize as forgeable names. If future
binary xlang libraries, incremental artifacts, LTO input, or object input are
supported, each carries the exact used-entry tuples, approval-snapshot
identity, and certificates. The consumer revalidates both the certificate and
current authenticated-snapshot membership. An unsupported form is rejected rather than treated
as ordinary code. A linker symbol or runtime helper name never mints facts;
helper bytes and ABI are certificate- or TCB-bound. Ordinary FFI cannot target
a privileged helper identity. Runtime symbol aliasing, stale certificates,
cache replay, root mismatch, and helper substitution are gate failures.

The bootstrap graph can be acyclic, but that is not a trusting-trust proof.
Stage 0 remains explicit TCB. It embeds the reference-policy and verifier roots;
stage 1 and stage 2 embed the same identities and revalidate all used entries.
A self-hosting fixpoint must cover those embedded roots, the verifier identity,
and the synthesized primitive table, not only emitted program IR. Stage 0 does
not leave the TCB without a separate diverse-compilation, independent-verifier,
or machine-proof result.

### 3.3 Why this gate was selected

It demonstrates zero ordinary-source privilege-admission spellings while
retaining explicit per-entry authority:

- **Source and artifact isolation:** accepted ordinary inputs cannot request or
  forge privilege relative to the trusted installation.
- **No flag:** a build mode cannot silently turn ordinary source into trusted
  source.
- **External generativity:** a certified, human-approved third-party frame can
  enter without waiting for a new standard library or compiler release.
- **No derived-container-library dependency:** the toolchain ships a normative
  capability core, verifier, and necessary allocator/runtime frames, but need
  not ship `Vec`, `HashMap`, strings, or other derived policies.
- **One audit path:** special type identity, intrinsic lowering, runtime frames,
  and compiler-known operations use the same package and ledger schema.
- **Per-semantic accountability:** one gate does not collapse distinct fact or
  effect obligations into one review item.
- **Acyclic bootstrap identity:** stage 0 can pin the verifier and policy roots
  independently of the xlc artifact it compiles, without claiming that a
  fixpoint removes stage 0 from the TCB.

This instantiates one possible admission mechanism under the direction already
present in GATE-1 and LEDGER-1. It does not by itself delete or merge the
current semantic categories of unsafe regions, FFI frames, and trusted
primitive imports. Any such reconciliation is a separate specification
decision. Per-entry proof, effect, fact, and trust obligations remain distinct.

### 3.4 Why no keyword is preferred

A keyword such as `box` is appropriate only when the operation is part of the
ordinary language. Privilege itself should not be part of the ordinary
language. A parser-recognized but context-restricted keyword creates questions
about who may enable the context, how tools parse it, whether macros or copied
source can reproduce it, and whether a compiler flag changes its meaning.

The selected gate direction instead gives ordinary code zero privilege-admission
spellings. Ordinary code necessarily sees safe capability operations and may
need reviewable invariant/proof evidence; those are not privilege, and this
report does not claim they are invisible. A future library-defined singleton
unique heap owner might be built over an accepted public storage capability,
but current `box<T>` remains a builtin and this report has not derived its
replacement. A verified compiler lowering could optimize a future abstraction
without granting a second admission path.

## 4. Production-language evidence

The precedent survey supports private builtins as an admission mechanism, but
does not identify an existing language that also provides a Pareto-small,
proof-checked, generative public basis.

| System | Admission mechanism | What it establishes | Why it is not sufficient for xlang |
|---|---|---|---|
| Rust | [`#[lang = "..."]`](https://rustc-dev-guide.rust-lang.org/lang-items.html), [compiler intrinsics](https://doc.rust-lang.org/stable/core/intrinsics/index.html), and [special compiler-known types](https://doc.rust-lang.org/reference/special-types-and-traits.html#boxt) | Toolchain/core crates can provide compiler-known identities through explicitly unstable hooks; ordinary stable crates cannot rely on them. Compiler operations can be wrapped, and Rust `Box<T>` has non-replicable language semantics. | Rust has several admission forms. The official [Nomicon Vec construction](https://doc.rust-lang.org/nomicon/vec/vec.html) shows the conventional stable-Rust extension route using public `unsafe`; it is evidence of that design, not a proof that every conceivable Rust design must do so. Access control and manual unsafe invariants remain trusted. |
| Swift | A compiler-constructed `Builtin` module visible while building the standard library | The compiler book states that `Builtin` contains compiler types and intrinsics, is normally invisible, and user code does not interact with it directly ([compiler book, sections 2.5 and 3.3](https://download.swift.org/docs/assets/generics.pdf)). | This is the closest hidden-module precedent, not evidence of a security boundary: `-parse-stdlib` selects visibility. The builtin set is large, while ordinary custom buffer collections use facilities such as [`ManagedBuffer`](https://developer.apple.com/documentation/swift/managed-buffers) and unsafe pointer-level APIs. |
| GHC | Public `GHC.Exts` plus `MagicHash` | GHC exposes a documented ["raft" of primitive types and operations](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/primitives.html). | The gate is public and the primitive set is large. The [Safe Haskell trust model](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/safe_haskell.html) permits author-asserted `Trustworthy` modules rather than mechanically proving all low-level invariants. |
| Go | Tool-enforced `internal/` imports plus compiler/runtime recognition | The toolchain can enforce a path-based private package boundary ([Go 1.4 internal-package rule](https://go.dev/doc/go1.4#internalpackages)); the compiler has separate [intrinsic and lowering machinery](https://go.dev/src/cmd/compile/README). | `internal/` proves non-importability within a tree, not primitive soundness or privilege. The official [`go:linkname` documentation](https://pkg.go.dev/cmd/compile#hdr-Linkname_Directive) also exposes why link-time identity needs separate control. |
| OCaml | Compiler `%...` primitives and user-defined `external` bindings | The standard [`Array` implementation](https://github.com/ocaml/ocaml/blob/trunk/stdlib/array.ml) uses safe and unsafe primitives, and the [FFI manual](https://ocaml.org/manual/5.4/intfc.html) permits user-defined C primitives. | The ordinary extension route trusts C/runtime invariants; the official discussion of [`Obj`](https://ocaml.org/docs/memory-representation#obj-module-considered-harmful) describes type-safety and memory hazards. |
| Zig | Public `@builtin` operations | The [language reference](https://ziglang.org/documentation/master/) exposes pointer casts, `undefined`, memory operations, and runtime-safety control directly to ordinary code. | It demonstrates a unified spelling family, not private admission or proof-checked extension. |

Two negative findings are as important as the precedent:

1. None of the surveyed production systems simultaneously provides a private
   admission gate, a small semantic basis, ordinary safe-library generativity,
   and machine-checked low-level invariants.
2. Private builtins alone commonly move unsafe code into a larger trusted
   standard library. That result would not satisfy xlang's stated objective.

## 5. Literature evidence for the public basis

The academic systems do not supply a ready-made xlang design. Together they
show that the required components are feasible and expose their costs.

| Work | Relevant result | Design consequence |
|---|---|---|
| [ATS stateful views](https://open.bu.edu/bitstreams/2c20177c-44d5-498e-a7e0-3ac321b4f65f/download) and its [viewtype tutorial](https://ats-lang.sourceforge.net/DOCUMENT/INT2PROGINATS/HTML/c3321.html) | Linear views distinguish `T @ L` from uninitialized `T? @ L`; writing consumes the uninitialized view and produces the initialized view. Proofs can describe pointer arithmetic and array partitions. | Typed live/dead slot transitions and erased ownership proofs are feasible. ATS uses explicit views and proofs; it does not establish automatic recovery of arbitrary metadata invariants. |
| [Mezzo permissions](https://gallium.inria.fr/~fpottier/publis/pottier-protzenko-mezzo.pdf) | Affine and duplicable permissions support gradual initialization, memory reuse, typestate change, and ownership transfer. Permissions have no runtime representation. | State and ownership can be one erasable permission. Dynamic adoption shows that arbitrary membership may require runtime evidence when static proof is unavailable. |
| [Vault adoption and focus](https://www.microsoft.com/en-us/research/publication/adoption-and-focus-practical-linear-types-for-imperative-programming/) | Focus temporarily recovers linear authority from aliased structures; adoption handles members governed by an owner. | A restricted checked transition scope is a plausible zero-runtime-cost closure mechanism, but dynamic membership is not free. |
| [GhostCell](https://plv.mpi-sws.org/rustbelt/ghostcell/paper.pdf) | A generative brand and one token separate access permission from data; the token is zero-space and operations erase to casts. | Root identity and coarse permission can be static and zero-cost. GhostCell does not itself solve partial initialization, deletion, or exact live-subset cleanup. |
| [RustBelt](https://people.mpi-sws.org/~dreyer/papers/rustbelt/paper.pdf) | A safe interface for an unsafe implementation yields a library-specific verification condition; the work machine-checks important unsafe-library abstractions. | A private implementation is not sound merely because it is private. Every capability entry needs a proof obligation or an explicitly counted trusted assumption. |
| [Low-star](https://arxiv.org/abs/1703.00053) | A structured memory model and dependent verification produce memory-safe low-level programs; specifications and proofs erase, with competitive generated C reported for cryptographic code. | Proof cost can be paid at compile time without a runtime tax. The proof/checker/tooling cost remains real. |
| [Cogent](https://arxiv.org/abs/1601.05520) | Linear types justify in-place update and a proof-producing compiler for efficient C without a garbage collector. | Unique ownership can support efficient systems code and certification, but restricted languages still need verified foreign components for missing low-level operations. |
| [Steel](https://fstar-lang.org/papers/steel/) | Dependently typed concurrent separation logic supports verified sequential and concurrent linked structures using automated and interactive proof. | A sufficiently expressive proof model can cover later shared and concurrent families, but it is not a small ordinary type checker by default. |
| [Proof-carrying code with untrusted proof rules](https://people.eecs.berkeley.edu/~necula/ISSS02/isss02.pdf) | A producer can carry a machine-checkable safety proof, and high-level proof rules can themselves be removed from the trusted base by proving that they imply a reference memory-safety policy. | A sealed admission verifier can accept certified implementations without permanently trusting every high-level proof rule, but the verifier and reference machine semantics remain in the TCB. |
| [Cyclone regions](https://www.cs.cornell.edu/projects/cyclone/papers/cyclone-regions.pdf) | Region capabilities provide explicit allocation and lifetime structure. | Regions help allocation lifetime and borrow provenance, but do not alone prove which slots contain initialized values. |

The surveyed literature supports one central candidate direction:
ownership/state relations can be represented by proofs that are checked and
erased, avoiding a mandatory runtime mirror in the systems studied. It does not
show that arbitrary library invariants can be inferred automatically, that the
combined xlang design is already feasible under D1a/META-5, or that proof
generation is cheap. ATS uses explicit views; Low-star and Steel co-develop
programs and proofs; proof-carrying code moves proof construction to the
producer. If xlang later consumes an out-of-tier proposition as safety or fact
authority, any proof-bearing fallback must count proof-generation, checking,
diagnostic, and artifact cost. Ordinary algorithm invariants need no such
certificate unless explicitly promoted. A private unchecked core and a
universal runtime bitmap remain controls rather than selected defaults.

## 6. Semantic lower bound

The repository evidence cannot prove a lower bound on keyword or opcode count.
It does prove that a viable basis must account for independent semantic
authority in all of the following dimensions:

1. **Backing resource:** generative root identity, layout, capacity, acquisition
   failure, version, and exactly one release.
2. **Typed liveness:** allocated storage may contain no `T`; initialization,
   move-out, replacement, swap, relocation, destruction, and reuse conserve
   affine values.
3. **Aggregate live set:** prefix, interval, two-range, sparse, dependent, and
   transient-hole states use their contract-required metadata rather than one
   universal representation.
4. **Normal-exit closure:** every fallthrough, return, break, propagation,
   callback stop, and recoverable failure leaves a valid owner or performs an
   exact verified disposal. Affinity alone is insufficient.
5. **Borrow authority:** provenance, disjoint footprints, result reborrows,
   cursor lifetime, and invalidation are not consequences of slot liveness.
6. **Failure and behavior boundaries:** allocation, callback, comparison,
   hashing, cloning, and early-stop branches need exact commitment and owner
   disposition.
7. **Fact coherence:** metadata that authorizes a load or optimization is tied
   to root, version, producer, consumers, transfer, and invalidation.
8. **Ordinary-library abstraction:** representation sealing, generics, direct
   monomorphized behavior calls, and non-special-cased source are necessary for
   unseen structures.

Putting all eight behind one name does not remove any of them. The candidate
resource-transition algebra gives them one compositional proof model, subject
to the explicit certificate and closure costs below.

### 6.1 The safety-authority trilemma

One lower bound survives every reviewed candidate, but its scope must be
precise. When a proposition is consumed to authorize a payload read, move,
drop, release, unique borrow, or check-eliding optimizer fact, that proposition
must be established in one of three ways:

1. **static proof:** the checker or a producer-supplied certificate proves it;
2. **runtime validation or enforcement:** ordinary data, tags, bounds checks,
   hardware protection, or other dynamic mechanisms establish it before use;
   or
3. **trusted semantic authority:** a privileged implementation or rule asserts
   it.

Universal shadow state is only one expensive instance of the second route.
On-demand validation against the representation's existing metadata can be
both safe and efficient. Representation choice can also keep the safety
topology simple even when the abstract algorithm is complex.

The trilemma does **not** make every algorithmic invariant a machine-proof
obligation. Sortedness, key uniqueness, hash-table lookup semantics, graph
reachability, and similar properties remain ordinary program correctness unless
the library explicitly exports them as a checked contract or optimizer fact.
They are tested and reviewed like other logic. The safety checker must prove
only the ownership, initialization, provenance, destruction, race, overflow,
and fact propositions it consumes.

H-STORE demonstrates the distinction. In its dense/sparse representation,
`p < n` plus prefix liveness is sufficient to read `dense_key[p]` and
`value[p]` safely. The subsequent `dense_key[p] == k` check determines whether
the lookup is semantically a hit; uniqueness and position coherence establish
map correctness, not initializedness. The currently frozen requirement that
H-STORE exercise a general `ST-SPARSE` mechanism may therefore overconstrain
the safety substrate and must be re-adjudicated before it is used as a basis
gate.

The selected requirement is narrower than general proof-carrying
generativity: no writer assertion may create safety authority, and every
consumed proposition needs one of the three explicit dispositions above.
Bounded deterministic reasoning should be tested first. Universally available
proof-bearing ordinary modules remain a feasibility fallback if a required
efficient shape genuinely needs relations outside that tier; topology-specific
privilege is not the default fallback. Foundational PCC and concurrent
separation logic show that such a route can exist, but no evidence yet makes
their exact logic necessary, minimal, or acceptable under D1a and META-5.

## 7. Bounded sequential storage-basis candidate

### 7.1 Abstract model

The smallest currently falsifiable candidate is limited to sequential,
unique-owner typed storage of affine `T`. It excludes stored borrows, shared
ownership, pinning, custom allocators, FFI, external resources, and concurrent
access. It has one runtime block owner and one exact erased slot-state
permission:

```text
Block(rho, T, C)
Slots(rho, T; Dead, Live)
```

`rho` is a fresh root identity and `C` is checked capacity. `Block` is an
affine runtime owner of backing storage. `Slots` is exact-use proof authority;
it has no runtime representation. A complete block state covers `[0,C)`.
Separating composition proves disjoint authority and conserves every affine
payload.

Arbitrary mathematical subsets are not part of the bounded candidate. Its
first-slice live-set grammar is finite and decidable:

```text
LiveShape(C) := Empty
              | Full
              | Prefix(n)
              | Interval(a,b)
              | Pair(a,b,c,d)

0 <= n <= C
0 <= a <= b <= C
0 <= a <= b <= c <= d <= C
```

`Pair` denotes two disjoint live ranges `[a,b)` and `[c,d)`; dead slots are the
complement. This grammar covers full, empty, dense prefix, owning interval,
gap, ring-wrap, and operation-local hole states. It deliberately does not
cover an arbitrary sparse control-byte relation. Extending it one topology at
a time risks recreating a privileged owner catalog; replacing it with general
predicates requires a separately selected and fully counted proof-carrying
logic.

A nominal ordinary-library owner would have to seal a block, one complete
shape permission, and its runtime endpoints while existentially hiding `rho`.
Fresh generative identity and existential or abstract-type sealing are
load-bearing, not notation. Current xlang does not yet provide the required
ordinary-library abstraction mechanism, so this is a blocking prerequisite.

The valid-value invariant is:

```text
0 <= i < C and i in L     => exactly one owned T exists at slot i
0 <= i < C and i not in L => no T exists at slot i
```

The nine schematic transition-role families are:

```text
acquire_dead_trap<T>(C)
  -> exists rho. Block(rho,T,C) * Slots(rho,T; Empty)

release_dead(
  own Block(rho,T,C) * exact Slots(rho,T; Empty))
  -> ()

split_at(endpoints,
  exact Slots(rho,T; Shape))
  -> two exact disjoint interval permissions justified by LiveShape

join(P1, P2)
  -> one exact LiveShape permission, only for a grammar-valid disjoint union

init(i,
  address Block(rho,T,C), exclusive Dead(rho,T,i), own T)
  -> Live(rho,T,i)

take(i,
  address Block(rho,T,C), exclusive Live(rho,T,i))
  -> Dead(rho,T,i) * own T

borrow(i,
  address Block(rho,T,C), locked Live(rho,T,i))
  -> root-and-version-tied borrow T; Live returns when the borrow ends

swap(i, j,
  address Block(rho,T,C), exclusive Live(rho,T,i) * exclusive Live(rho,T,j))
  -> Live(rho,T,i) * Live(rho,T,j)

replace(i,
  address Block(rho,T,C), exclusive Live(rho,T,i), own T)
  -> Live(rho,T,i) * own T

focus(owner<R>) { body }
  -> owner<R'>, only if every normal exit repacks one complete valid owner
```

`acquire_dead_trap` follows current OP-9 rather than silently selecting
recoverable allocation. A future recoverable form needs its exact offered-owner
and failure-commitment contract under OD-1. Existing fully initialized heap or
inline storage separately needs a checked `adopt_full` structural rule; it
cannot be derived by allocating dead storage, and it is not selected here.

`split_at` and `join` are proof-only. `init` performs one typed write; `take`
performs one typed move-out. Their footprint permission is exclusive, and both
are rejected while any incompatible borrow rooted in that slot is live. A
borrow locks the corresponding live permission until the borrow ends.
`swap` requires two distinct disjoint live slots and no overlapping borrow. It
exchanges their affine payloads as one checked transition while preserving the
same live shape. It is explicit because decomposing an arbitrary nonadjacent
swap into two takes can create three live intervals, outside the bounded
`Pair` grammar.
`replace` similarly exchanges one live slot with an offered owner as one
shape-preserving transition and returns the displaced sole owner. Decomposing
replace through a take may leave the bounded grammar when the slot is interior
to either range of `Pair`.

`focus` is intended as a flow judgment, not a runtime flag, callback, or
repair-on-drop token. Fallthrough, return, break, propagation, callback stop,
and recoverable failure must repack a complete valid owner. Trap paths need no
recovery under EFF-4 but may perform no invalid access before abort. Capture,
calls, loops, callbacks, branch-specific postconditions, and existential
repacking still require exact static rules; until then `focus` is a requirement,
not a derived mechanism.

Relocation is take at the source plus initialize at the destination only when
the operation's intermediate and successor footprints remain in `LiveShape`;
the witness traces use boundary or hole movement that satisfies this condition.
Arbitrary relocation outside it remains blocked. Growth is acquire, permitted
relocation, and release. Clear is repeated take and structural drop.
These derived operations may receive optimized compiler lowerings only after
same-semantics proof. The sketch establishes their conservation equations, not
that current xlang can package or clean up the resulting owner.

### 7.2 Runtime metadata relation

The hard part is not allocation. It is proving that runtime metadata describes
the live set it authorizes. In the bounded candidate, only frozen arithmetic
relations between integer endpoints and one `LiveShape` are admitted, for
example:

```text
shape = Prefix(len)
shape = Pair(0, left, right, capacity)
```

The relation is proof state, not a runtime mirror. A fixed decidable checker
must validate every endpoint transition; writer assertion is never accepted.
Optional inference may construct evidence but is never a soundness premise.
The grammar, inference algorithm, diagnostics, artifact evidence, and
complexity limit remain unselected and count against D1a and META-5.

For the bounded shapes, the runtime metadata can be exactly what the algorithm
already needs:

- a dense sequence uses `len`;
- a deque uses head and length or two range endpoints;
- a gap buffer uses two endpoints;
- a B-tree node prefix may use its key count.

No universal bitmap, topology tag, generation field, or reference count is
forced by this abstract grammar. A state transition that updates endpoints and
payload must prove the joint successor relation before normal exit. Inline-
sparse payload occupancy, arbitrary pool slot liveness, and shared lifecycle
state are outside the bounded grammar when those relations authorize memory
access or destruction. Abstract hash, order, or graph invariants are not
automatically safety facts. A structure may instead use dense live prefixes,
fully initialized index metadata, and checked handles to keep its safety
topology within the bounded tier. Until an exact static design passes D1a,
META-5, and the held-out witnesses, this remains a research candidate rather
than a selected public basis.

### 7.3 Cleanup

Ordinary arbitrary finalizers remain rejected. A certificate is not executable
cleanup, and the nine transition roles do not decide who runs cleanup. Every
nominal storage owner nevertheless needs an executable path satisfying:

- every live affine value is taken and dropped exactly once;
- dead or moved slots are never read or dropped;
- backing storage remains live until the last payload is destroyed;
- backing storage is released exactly once; and
- every normal abandonment route has the same property.

Two honest mechanisms remain:

1. make the owner exact-linear and require an explicit consuming `close` or
   `dispose` on every normal path; or
2. add a verified disposer that the compiler invokes at an abandonment edge.

The first avoids a hidden finalizer but taxes source shape and error control
flow. The second is operationally a user-defined verified destructor even if
its proof is stored in an artifact. It therefore needs independently frozen
termination, recursion, control-flow, effect, allocation, trap, callback, and
nested-resource rules. A bounded alternative might let the compiler derive a
closed structural fold only for `LiveShape`: a prefix traverses `[0,len)`, an
interval traverses one range, and a pair traverses two. Such a fold may perform
only `take`, recursively approved structural destruction of `T`, and
`release_dead`; it may not call user callbacks or external effects. This still
needs a rule for `T` whose destruction owns a file, lock, or other external
resource.

Current STOR-3 supplies compiler-derived fixed cleanup and forbids user
finalizers, so neither generalized mechanism is current law. This report
records the normative fork instead of smuggling a disposer into `focus`.
Resource close operations with external side effects or failure remain
separate later-family contracts.

### 7.4 External and machine effects

The resource-transition proof model can describe files, sockets, locks,
atomics, and devices, but a storage transition cannot synthesize a system call,
an atomic instruction, or a target-specific register access. Those are
independently counted fixed-root entries or capability-capsule instances with
separate effect semantics. They use the same admission verifier and closed
resource/effect ledger. A capsule does not prove an opaque OS or device correct;
`TRUSTED_FOREIGN` records that boundary assumption. These entries are not
evidence that storage derives machine effects.

This is the honest minimum for a general systems language:

- one admission verifier and approval operation;
- one fixed reference proof/effect schema;
- `N` independently specified semantic authorities and `K` explicit unproved
  assumptions; and
- no ordinary-source privilege-admission spelling.

## 8. Conditional coverage hypotheses and sequential obligations

The first table is a conditional architecture sketch, not a completed type
derivation. A route is structurally promising when it can use the same
representation fields, asymptotic algorithm, allocation count, payload traffic,
and dynamic checks required by the corresponding contract if its certificate
is accepted and proof-only state erases. This does not claim compiler
throughput, proof-generation cost, or family closure.

The companion
[`SYSTEMS-DOMAIN-ADMISSION-MAP.md`](SYSTEMS-DOMAIN-ADMISSION-MAP.md) maps all 26
domains in the systems ledger to language, ordinary library, toolchain, or
independently counted gate entries. That map establishes an admission
architecture, not derivation from one storage primitive. The table below
expands the storage-bearing cases that most strongly constrain the candidate
algebra.

[`SEQUENTIAL-WITNESS-DERIVATIONS.md`](SEQUENTIAL-WITNESS-DERIVATIONS.md)
provides conditional transition traces for W-SMALL, W-GAP, H-FLATSET, and
H-STORE. It makes the inline-adoption, sealing, proof-logic, allocation-policy,
focus, and executable-cleanup blockers explicit.

| Demand family | Runtime representation and live relation | Conditional construction hypothesis | Required performance evidence |
|---|---|---|---|
| Fixed full buffer | Root, length; `L = [0,n)` permanently | Requires a still-unselected rule that adopts existing full or inline storage without reinitializing it. | The existing two-word path must remain byte-identical; the current candidate has not established this. |
| Dense growable sequence and heap | Pointer, length, capacity; `L = [0,len)` | Initialize at `len`; take at `len-1` or an index; shifts are relocations; growth is acquire/relocate/release | Same O(1) amortized append, O(1) pop, O(n) ordered shift, and geometric allocation envelope as an idiomatic vector. No dummy values or per-slot tags. |
| Ring deque | Pointer, head, length, capacity; `L` is one or two modular ranges | Partition the one/two ranges; initialize/take at endpoints; rotate or grow by relocation | Same O(1) endpoint operations and O(n) growth; no universal sparse bitmap. |
| Gap buffer | Pointer, two endpoints, capacity; `L = prefix union suffix` | Move the gap by paired take/initialize transitions; insert and delete at its boundary | Same O(1) local insert/delete and O(distance) gap movement; exactly the two endpoints already required. |
| Inline-small sequence | Inline root plus optional heap-root variant; prefix relation on the active root | Requires the same unselected adopt-inline rule, existential sealing of either active root, and one verified transition on spill. | Zero heap allocations through N is a gate; it is not yet derived by the nine-role candidate. |
| Hash table and sparse store | Dense payload/key prefixes plus fully initialized bucket or position metadata; alternative inline-sparse layouts remain controls | Bounds or occupancy validation first yields a safe dense index; abstract key/position coherence remains ordinary correctness unless exported as a checked fact | Expected O(1) lookup without a duplicate safety bitmap is structurally plausible. Inline-sparse payload performance remains a comparison, not a prerequisite silently assumed by H-STORE. |
| B-tree node and dependent prefixes | Key/value prefixes of n and child prefix of n+1 | Separate permissions for dependent ranges; split/merge relocate between exact footprints and roots | Same node layout and O(node width) split/merge traffic; no full initialization requirement. |
| Pool, arena, handles, graph, and ECS | Stable backing roots plus contract-required occupancy/generation/location metadata | Generative roots prevent cross-root ownership forgery; handles revalidate metadata; graph/ECS compose pool and dense transitions | Bare append-only identities retain their no-generation path. Recycling pays only its required metadata/checks. Payloads need no per-node heap allocation when the selected layout does not. |
| Singleton unique heap owner and recursive unique structures | One full slot per allocation or a region-owned set of full nodes | A future full-storage adoption rule could derive the singleton owner; recursive structures additionally require sealed recursion and exact cleanup. | A library-defined replacement for the current builtin `box<T>` is a specification change and has not been derived here. |
| Iteration, drain, retain, sort, and pipelines | Borrow or owning cursor plus a state-indexed remaining footprint | Partition current and remaining authority; each yield transfers or borrows exactly one footprint; early exit invokes verified closure/repair | No materialized intermediate collection is required. Proof tokens erase; algorithm-required cursor fields remain. |
| Bytes and strings | Dense byte storage plus a UTF-8 or other refinement proof | Checked validation seals a refinement; byte mutations preserve or invalidate it explicitly | Same byte layout as the underlying sequence; validation and boundary checks are paid only by the text contract. |
| Shared ownership and dynamic borrowing | Contract-required counts or borrow state plus a resource invariant | Duplicable observation handles refer to one generative resource; checked count/borrow transitions control unique access and disposal | `Rc`/`RefCell`-class structures pay their own counts and branches. Unique-owner structures pay none. Full closure needs its later family proof. |
| Concurrency and atomics | Contract-required atomic state and ordering | Atomics enter through separate fixed-root or capsule entries; a still-unselected concurrent proof system must justify publication, ordering, and disposal. | No atomic field need be imposed on sequential structures, but concurrency closure remains entirely separate. |
| Files, sockets, clocks, process, FFI, MMIO | Opaque resource identity and platform state | Gated machine entry plus checked resource transition, result, and effect contract | No container simulation is involved. The same gate admits the irreducible operation; each effect remains separately reviewed. |

The table is an obligation inventory, not a constructive derivation. It shows
where a compact runtime representation appears possible and where an
irreducible capability entry belongs. It does not establish that one proof
logic accepts every relation, that cleanup is expressible, or that the
generated code meets the protected baselines. Shared ownership, sparse
occupancy, relaxed atomics, pinning, custom allocation, async cancellation,
full text semantics, platform frames, and their proof systems remain open.

### 8.1 Held-out generativity

The current held-out identities remain the anti-special-casing test:

- W-SMALL requires an inline/heap transition without a hidden Copy or dummy
  value.
- W-GAP requires two live ranges and one hole without a universal bitmap.
- H-FLATSET requires an ordinary library to compose dense ownership,
  comparisons, relocation, and exact cleanup without importing a completed
  sequence.
- H-STORE tests whether its safety authority can remain dense-prefix plus
  checked indexing; any stronger sparse contract must justify why that route
  is insufficient rather than importing a privileged hash-table operation.

None of the witnesses appears to require a container name in the compiler or
capability root. That is a hypothesis worth preserving, not a passed result.
W-SMALL additionally needs full/inline adoption; W-GAP needs a bounded
two-range proof; H-FLATSET needs exact cleanup and behavior-call accounting;
and H-STORE needs its safety authority separated from its abstract map
invariant. The current nine-role candidate does not close all four.

## 9. Why the alternatives lose

### 9.1 Private arbitrary unsafe core

This is easy to build and can match any target performance, but it merely moves
Rust-style unsafe invariants into a privileged library. Every unforeseen data
structure either needs a new trusted implementation or cannot be written by an
ordinary library. Privacy does not prove soundness. It is retained only as a
complexity and performance control.

### 9.2 `Box` or one fully initialized allocation primitive

It cannot represent capacity greater than length for arbitrary affine values.
Box-per-element loses contiguity and allocation bounds. A boxed full array
requires dummy values, hidden `Default`/`Copy`, or per-slot tags. It fails
dense, gap, sparse, and dependent-prefix witnesses.

### 9.3 Raw allocation only

Bytes plus capacity do not prove valid `T`, exact drop, ownership conservation,
provenance, disjointness, failure commitment, or optimizer facts. Public raw
access violates the no-unsafe law; private raw access fails ordinary-library
generativity.

### 9.4 Universal runtime bitmap

It can safely encode many live sets, but charges O(capacity) metadata and an
occupancy branch to fixed, dense-prefix, deque, and other compact topologies.
It violates G-15 and the protected B-FIX/B-P2 paths.

### 9.5 Closed topology-owner catalog

`DenseOwner`, `SplitOwner`, `SparseOwner`, and similar sealed types can be safe
and efficient for anticipated structures. They are a useful fallback and
implementation technique. As the foundational answer, however, they turn the
language core into a growing catalog of topology names. An unforeseen topology
would require a new privileged owner unless it can be reduced to existing safe
representations, runtime validation, or checked proof. That is why the catalog
remains a control rather than the selected foundation.

### 9.6 General proof-carrying library logic

ATS, Low-star, Steel, and proof-carrying code demonstrate relevant
expressiveness, but moving proof terms into artifacts does not make their
logic free. A general separation or dependent logic still adds normative
rules, a checker, certificate production, artifact size, diagnostics, and a
human-review boundary. It may be the only generative route for arbitrary
safety-relevant sparse, recursive, or concurrent invariants when representation
and runtime validation are insufficient, but abstract algorithm correctness
does not create that requirement. This report has not selected an exact logic
or shown that one is necessary under D1a and META-5. Calling it an
implementation detail would hide its cost.

## 10. Structural performance hypothesis and limits

The abstract model avoids mandating one universal bitmap, topology tag,
generation field, or reference count. If a future design also proves that:

1. brands, separation permissions, proof terms, and state identities erase;
2. runtime metadata is chosen by the library's contract, not by the universal
   basis;
3. safe wrappers monomorphize to direct operations;
4. checked transitions lower to the same loads, stores, branches, allocation
   calls, and payload moves as the selected algorithm;
5. a proof-derived fact may remove a check, but facts-off retains safety and
   accepts the same program; and
6. fixed/full and append-only protected paths instantiate no partial, sparse,
   generation, sharing, or policy state.

then a same-layout and same-asymptotic implementation is plausible. None of
those conditions has been measured for this candidate. Transition validation,
wrapper lowering, disposal cold paths, code size, proof checking, and
certificate production can still add cost. The present result is therefore
only that no universal runtime metadata tax has been shown necessary. Exact
IR, protected-path byte identity, operation counts, checker time, and runtime
measurements remain mandatory before any no-tax or performance claim.

## 11. Hostile boundary conditions

Any later design fails this report if it permits one of the following:

- a source flag, module path, package name, attribute, or copied manifest to
  grant privilege;
- one opaque "magic" entry whose unledgered branches contain multiple effects
  or fact channels;
- ordinary code to assert that metadata implies liveness;
- a `mark_initialized`, `set_len`, `assume`, or manual release operation not
  backed by checked ownership transitions;
- a protocol token that may be abandoned while leaving a partially rebuilt
  owner;
- a cleanup path that trusts client metadata without the checked relation;
- same-address reuse preserving root/version facts;
- a completed container or recognized source name as a held-out dependency;
- a universal topology, generation, sharing, or occupancy tax on a weaker
  protected path;
- a privileged core that grows by one operation whenever a new data structure
  is encountered; or
- calling an unproved standard-library implementation "safe" solely because
  ordinary users cannot access its internals.

## 12. Trust and verification disposition

The sealed capability admission verifier is an admission mechanism, not a
soundness proof. The preferred disposition for a capability entry is:

1. a fixed reference machine/effect semantics;
2. a machine-checkable certificate that the lowering or runtime helper
   satisfies its resource/effect contract;
3. a small certificate checker and dependency-cone identity;
4. independent backend and facts-off parity tests; and
5. hostile review for every fact-producing entry.

Where a machine proof is not yet practical, the entry remains an explicit TCB
assumption with a smaller claim and cannot mint optimizer facts beyond its
reviewed contract. A capsule or root digest does not convert trust into proof.

## 13. Research disposition

The research supports one architectural selection and one explicit non-
selection:

- **Selected gate direction:** one sealed capability admission verifier with a
  fixed compiler-embedded root and capsules admitted through one authenticated
  signed approval-snapshot predicate. Ordinary source and dependencies have
  zero privilege-admission spellings.
- **Selected authority scope:** every proposition consumed as safety, resource,
  or optimizer-fact authority requires static proof, runtime validation or
  enforcement, or an explicitly ledgered trusted authority. Ordinary source
  can use the first two routes but cannot mint the third. Abstract container
  semantics remain ordinary correctness unless explicitly promoted to a
  checked contract or fact.
- **Not selected:** a universal public proof or storage foundation. The
  nine-role sequential model is a falsifiable candidate for a bounded
  unique-owner slice, not evidence that arbitrary sparse, shared, concurrent,
  recursive, or external-resource structures are derivable.
- **Derived container standard library:** not required and not part of the TCB.
  Containers remain intended to be user- or AI-authored ordinary libraries,
  but that intent is not yet proved achievable for the complete systems
  envelope. The toolchain still contains the capability root, verifier, and
  required runtime frames.
- **Machine effects:** distinct per-operation fixed-root or capsule entries use
  the same gate and ledger; storage does not derive them.

The selected gate still implies future specification work. Current PROG-1 has
no module loader, so the proposed authority is an unspellable compiler identity,
not a current private module. LEDGER-1's existing unsafe-region, FFI-frame, and
trusted-import categories are not deleted by this report; unifying their
admission path would be a later normative decision. The current builtin
`box<T>` also remains unchanged.

The public basis remains blocked on:

- source spelling for a transition scope or any explicit proof-bearing fallback;
- exact logical live-set language and inference boundary;
- certificate format and internal proof checker if bounded reasoning proves
  insufficient;
- representation and existential sealing of a fresh public storage owner;
- whether any required efficient sequential shape needs a general proof-
  carrying fallback after runtime validation and representation alternatives
  are exhausted;
- a full/inline-storage adoption rule;
- exclusive-access and live-borrow rules for initialization and take;
- allocation-failure disposition, which remains OD-1 rather than being fixed
  by this report;
- exact-use focus semantics for calls, loops, callbacks, and all normal exits;
- either exact-linear manual close or a separately selected verified disposer,
  including termination, effects, recursion, and nested-resource behavior;
- whether common topology owners are library conveniences or compiler-provided
  wrappers;
- production compiler lowering; and
- any language, specification, or default-teaching change.

These are not mere implementation details. They determine whether the public
basis is expressive, sound, reviewable, and efficient. The honest answer to the
present expressibility question is therefore split: the gate architecture is
credible; complete ordinary-library derivability has not yet been proved.

## 14. Required staged next research

One all-systems Proof Logic Lock would mix unfrozen semantic strata and fail by
construction. The next work must be staged.

### Stage A: Gate Authentication Lock

Freeze and attack the one acceptance predicate in Section 3: embedded base
entries, the pinned approval-root key, canonical signed snapshot, protected
epoch, exact scopes and dispositions, ordinary-artifact revalidation, cache and
binary replay, rollback, revocation, bootstrap identity, and the ordinary-
proof versus capsule boundary. This stage selects no public storage operation.

### Stage B: Bounded Sequential Safety Kernel Lock

Freeze only sequential unique-owner safety:

- the bounded `LiveShape` grammar and nine transition roles;
- generative opaque-owner sealing and root/version invalidation;
- focus rules for calls, loops, callbacks, and every normal exit;
- full and inline adoption;
- exact-linear close versus restricted verified disposal;
- current trap allocation plus an explicit OD-1 boundary;
- safety-only H-STORE treatment; and
- facts-off identity and protected-path structural/performance protocols.

The same rules, with no container-specific additions, must cover W-SMALL,
W-GAP, H-FLATSET, and the dense-prefix safety route for H-STORE. The existing
H-STORE `ST-SPARSE` requirement must be re-adjudicated before it can reject this
route. Stage B freezes, but does not execute, a future same-contract comparison
of the dense/indexed hash-store shape against idiomatic Rust with a
preregistered margin and attribution protocol. A later separate authorization
would be required to construct or measure either side. Only a formal
counterexample or later authorized measurement showing that the extra index or
indirection causes the performance failure may justify Stage C or a bounded
inline-sparse extension; intuition alone may not.

### Stage C: Conditional general-predicate lock

Open this stage only if Stage B finds a formal counterexample or later
separately authorized measurement of a required efficient shape proves that
bounded proof, runtime validation, and an alternative representation are all
insufficient. Then price a foundational proof-bearing fallback, including
its normative logic, checker TCB, certificates, resource limits, diagnostics,
artifact identity, and proof-production burden. Do not prepay this cost from an
abstract-map invariant that the safety checker never consumes.

### Stage D: Later semantic families

Recursion, shared ownership, dynamic borrowing, relaxed atomics, and
concurrency receive separate locks after their language and memory-order
semantics exist. Sequential evidence cannot discharge them.

Stages A and B are paper/static-design research only. They authorize no
compiler feature, standard library, container candidate, language change,
production fact channel, or execution.

## 15. Owner review questions

1. Accept the sealed capability admission verifier--embedded base root plus one
   authenticated signed approval-snapshot predicate for scoped capsules--as the
   sole research direction for privileged admission?
2. Confirm the authority boundary: every proposition consumed for safety,
   resources, or optimizer facts must come from static proof, runtime
   validation/enforcement, or explicit ledgered trust; ordinary source cannot
   mint trust, and abstract container semantics remain ordinary correctness
   unless promoted to a checked contract or fact?
3. Ratify the ordinary-artifact boundary: proof over existing public semantics
   is unprivileged and available to every producer; only a new machine, helper,
   lowering, backend, ABI, device, OS, or foreign semantic edge requires an
   authenticated capsule?
4. Authorize only Stages A and B in Section 14, retaining the nine-role model
   as a bounded candidate and opening Stage C only if evidence establishes its
   necessity?
5. Keep G0 and the dense lock as completeness and adversarial tests, with OD-0
   through OD-5, candidate construction, language/specification changes, and
   implementation still suspended?
