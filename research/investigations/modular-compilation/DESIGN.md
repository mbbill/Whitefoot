# Modular compilation with incremental verification and optimization

This is a proposed architecture, not implemented language behavior. Its
requirements are independent module compilation, incremental work from source
checking through optimized object generation, ordinary final linking, and
runtime optimization that is not
artificially limited by source module boundaries. The active specification and
live design tree remain authoritative until amended. The inspected baseline is
`f3cf41d42cf6324a83c1de0b28e6c0d4e6ff9da8` (kernel v0.62).

This investigation owns the architecture, its alternatives, external evidence,
and selection criteria. Keep it as the grounds for the eventual compiler and
language decisions; revise it in place when evidence changes the proposal.
Remove it only after its grounds are superseded or preserved elsewhere and it
contains no unique useful evidence.

## Required outcome

1. A module's unchanged bodies do not need to be parsed, resolved, type-checked,
   ownership-checked, or proved again merely because another module changed.
2. Reuse is conditional on every semantic dependency remaining valid. Cached
   and clean checking have the same acceptance and rejection judgments. No
   timeout, cache miss, scheduling choice, or optimization result is evidence.
3. Public contracts are verified against implementations. An interface file,
   object file, fingerprint, or writer assertion is not proof of that relation.
4. Generics retain concrete specialization, source checking, and direct calls.
   Module boundaries introduce no dictionaries, boxing, or dynamic dispatch.
5. Inlining, specialization, constant propagation, layout optimization, and
   proof-derived optimizer facts remain available across module boundaries.
6. The architecture tracks semantic dependencies separately from dependencies
   introduced by optimization. An optimized caller can need new machine code
   without needing a new source proof.
7. Final native linking may run in full. Reuse unchanged objects and runtime
   construction, preserve ordinary link optimizations, and include final link
   time in end-to-end measurements. An incremental native linker is not required.

The performance objective is the best runtime implementation supported by
measured evidence for the target workloads, with practical incremental builds.
It is not a claim that a compiler can decide the globally fastest equivalent
program. Reusing stale inlined code, permanently disabling cross-module
optimization, and making every edited build a whole-program proof run all fail
the objective rather than providing alternative completion levels.

## Existing boundaries

- [PROG-1/2](../../../spec/kernel-spec.md#11-programs-closed-world) currently
  form one ordered source bundle, without modules or separate compilation.
- [FN-1/2/6/9](../../../spec/kernel-spec.md#8-functions-generics-contracts)
  already separate callable contracts from implementations, require symbolic
  template and concrete-instance checking, constrain instantiation cycles, and
  publish postconditions only after a recursive component has been checked.
- [DIAG-2](../../../spec/kernel-spec.md#12-diagnostics-and-checked-compilation-toolchain-floor)
  binds lowering authority and proof identities to one private checked program.
- [TYPE-6](../../../spec/kernel-spec.md#4-types) uses whole-unit declaration
  identities and visibility. Stable module-qualified identities will need to
  replace source-bundle ordinals as persistent semantic identity.
- The [driver](../../../compiler/src/driver.rs),
  [semantic checker](../../../compiler/src/semantic/check.rs), and
  [LLVM emitter](../../../compiler/src/backend/emitter.rs) currently build one
  checked inventory and one LLVM module. This design changes those boundaries;
  splitting LLVM output alone does not meet the required outcome.
- [GrowVector](../../../lib/containers/grow-vector.wf) exposes representation
  paths in both contracts and effects. Source privacy, proof visibility, and
  physical layout visibility need separate definitions.

## Architectural direction

Use explicit source modules and one persistent dependency graph whose queries
own parsing, exported declarations, checking, proof publication, instantiated
bodies, target layout, optimization planning and code generation, followed by
an ordinary final link over the selected objects.
The cold build evaluates the same queries as an incremental build. There is
one checker and one lowering path.

Each checked callable publishes its semantic boundary separately from its
implementation evidence. Consumers depend on the semantic value they read;
final composition also requires the currently selected implementation's valid
evidence. Changes to evidence alone must not cause transitive source reproof
when the consumer's propositions and their availability are unchanged.

Persist small module and function summaries. Load a body only for a dirty
checking query, a concrete generic instantiation that needs it, or an optimizer
that imports it. Source module identity, verification recursion groups, and
LLVM code-generation partitions serve different purposes and are not forced
to coincide.

## Source modules and their interfaces

### Membership and naming

The proposed surface has an explicit `module` header, explicit `import`
headers, and a `pub` modifier on exported declarations. The pub modifier precedes the existing declaration modifiers, and a public
readonly field begins with pub readonly. A module contains one
or more ordered source records supplied by the build. Each record states the
same module path; imports are file-local and precede declarations. A build
binding resolves each imported path to exactly one module identity and source
or checked-artifact selection. Directory enumeration, search order, ambient
package installation and the importing file's location never select meaning.
Package acquisition is outside this language mechanism.

Conceptual spelling, not source accepted by the current grammar:

```text
module app;
import collections::vector;

pub fn run(...) ... {
  ... collections::vector::grow_vector_new::<u64, 4096>() ...
}
```

Cross-module references use the complete imported path. There is no wildcard
import, implicit transitive import, source alias or overload selection. The
build can bind dependency versions under distinct explicit root names; nominal
identity includes that selected dependency identity, not just its printed name.
An implementation-private revision is not itself a new nominal identity.

Top-level functions remain visible throughout their own module. Other local
declarations retain TYPE-6's visibility points in the module's ordered records.
An imported module exposes a completed export table, so importer visibility
does not depend on exporter files preceding importer files. Imported names
are qualified and do not shadow local names. Prelude names retain their
ordinary reservation and visibility. The existing collision domains apply
inside each module; same-spelled declarations in distinct modules are distinct.

Qualification must reach every affected grammar role together: types,
constructors, callees, function arguments, interface/binding groups, named
constants, match variants, enum payload projections, destructuring and
postcondition result routes. Fixed operation-table names and numeric bounds
do not gain module-dependent lookup. The existing `::` before a function's
generic arguments remains; qualified-name productions must be factored and
checked by the specification's strong-LL(2) generator. This selects the naming
rule, not an untested complete EBNF patch.

The build selects a module-qualified entry through PROG-3's ordinary boundary.
`program no_heap;` belongs to the selected root module, before its ordinary
declarations after its headers. It constrains the complete selected dependency
closure, including concrete instances and linked prelude definitions; moving
an allocation into a library does not hide it.

### Visibility, representation and proof paths

Top-level declarations and struct fields are module-private unless exported.
Public type positions, including callable types, public fields, enum payloads,
generic bounds and groups, must have a public naming closure. A public struct does not grant access to private fields. Its constructor
and whole destructuring are available externally only when all required fields
are public and the existing `opaque` rule permits them. Ordinary non-opaque
construction remains available inside the module. `opaque` keeps its existing
meaning even in its declaring module; it is not redefined as privacy. A public
enum exports its complete variant and payload schema so ordinary exhaustive
matching remains available. Partially hidden enums are not introduced here.

Privacy controls source access, not information available to the compiler.
Compiler interfaces retain private layout and release descriptions needed for
by-value representation, generic checking, cleanup and optimization. No forced
pointer indirection, exported destructor call, stable public layout or runtime
dictionary follows from privacy. Dropping still requires the ordinary release
capability; private linear fields cannot be discarded through an external
destructor-shaped escape.

A contract written in a module may name that module's private field paths.
They remain in its generated semantic interface with ordinary types and stable
identities. A caller cannot write an otherwise-private selection, but the
checker can substitute and transport the published relation through the same
CALL rules. A public `length` function can publish its result equal to a private
window's length; a caller's branch on that result can then discharge another
exported function's requirement over the same length. No new fact constructor
or unchecked abstraction axiom is introduced.

This gives access encapsulation without promising representation-independent
proof interfaces. Changing a private projection named by an exported contract
changes that semantic interface and can require client reproof. A private
layout change read only by code generation invalidates layout consumers.
Representation-independent logical views are a separate language capability
needing their own proof and performance grounds. This proposal exposes the
actual dependency and does not prevent that extension.

### One written interface, several generated projections

The exported WF declaration is the interface authority. Generate its machine
projections rather than handwritten headers that can drift. Collect cyclic
module headers before body checking; a collected signature is not a verified
postcondition and grants no lowering authority.

| Projection | Content | Readers |
|---|---|---|
| Name surface | Exported identities, kinds, visibility and signatures | Lookup and declaration formation |
| Semantic boundary | Modes/types, capabilities, effect paths, normalized requires/ensures, constants and bounds | Source checking and proofs |
| Verification evidence | Checked implementation identity, obligations, derivations and proof dependencies | Composition and audit |
| Generic template | Resolved body, symbolic check and explicit parameters | Concrete instantiation |
| Physical description | Target layout, ABI, release shape and target obligations | Lowering and code generation |
| Optimization description | Checked IR, call edges, allocation facts, proven target facts, cost/profile summaries | Planning and body import |

One artifact can hold these sections with lazy access and independent keys.
These are not six public file formats or six checking paths. The format is
compiler-version-private; cache compatibility is not a language guarantee.

## Persistent computation and identity

### Query boundaries

Use deterministic dependency-recording queries with explicit input values.
These families name responsibilities, not a proposed public Rust API:

| Query | Relevant inputs | Reusable output |
|---|---|---|
| Source formation | Exact source bytes, grammar/spec identity | Tokens, canonical tree, source map |
| Module surface / lookup | Headers, selected imports, lookup role and spelling | Stable declaration or diagnostic |
| Contract / type shape | Resolved declaration, arguments, capabilities and projections | Normalized semantic boundary |
| Template check | Symbolic body, bounds, callee boundaries and summary availability | Symbolic checked body |
| Concrete body check | Body, complete substitution and semantic query results | Typed body, ownership/effects and obligations |
| Proof component | Current component membership, obligations and predecessor summaries | Verified clauses and derivations |
| Allocation / call summary | Current local seeds and call edges | Specified fixed-point summaries |
| Layout / lower | Checked body, target, relevant layouts and release descriptions | Target-qualified IR |
| Optimization plan | Summaries, visibility, prevailing definitions, profiles and policy | Imports and per-partition decisions |
| LLVM backend | Own/imported IR, decisions, toolchain and target settings | Optimized object and remarks |
| Final link action | Selected objects, runtime, linker/options and entry | Executable; ordinary full link when inputs change |

Module is the source/distribution boundary; a function or concrete instance is
the ordinary body-check boundary; a recursive component is an atomic proof
publication boundary. Query granularity is not forced to whole modules. An
edited file can be reparsed while unchanged item values stop downstream
invalidation. Token-level editor parsing is not necessary to avoid checking
untouched files and bodies.

Re-evaluate affected queries and compare their result values before invalidating
consumers. A changed body with the same verified callable boundary does not
change ordinary call-site checking inputs. Track negative lookup results,
export-set queries and candidate eligibility too: a previously absent name or
unprofitable inline candidate can become relevant. Do not place the hash of
every imported module's entire source in every consumer key.

Separate stable identity from revision. Declaration identity uses selected
module identity, domain and name; item-local node identities plus the current
source map recover diagnostic locations. Concrete instances add the complete
normalized type, const and function argument vector. Dense FunctionId/NominalId
values and whole-program NodePaths can remain in-memory indices, never
persistent identity. Adding an unrelated declaration must not rename all later
functions or LLVM symbols.

Documentation/location edits update canonical formation and source maps.
Reusing proofs requires unchanged semantic syntax and current remapping of
every retained location. Cached errors must not point to old byte offsets.
Failures may be cached only with complete dependencies and valid remapping.
Select diagnostics using a deterministic stage/module/item order, respecting
DIAG-1's stage and within-node rules. Worker completion order never selects the
reported violation. Source-map changes invalidate diagnostic rendering even
when they do not invalidate a semantic result.

### Cache authority and publication

A cache stores ordinary checker results, not independently asserted facts.
Schema/compiler identity, specification identity, selected dependencies and
actual query inputs must match. Target-dependent results also bind layout,
ABI, CPU features, LLVM/runtime versions and relevant options. Profiles bind
their content and mapping. Scope these inputs to consumers: changing an
optimization policy does not invalidate every source proof.

Digests locate candidate records; they are not proofs. Acceptance-bearing
identity checks retain canonical inputs and resolve digest collisions by
equality, so different contracts cannot become identical through a collision.
Records originate inside the compiler's trusted build domain. Externally
supplied WF artifacts without that provenance need source checking before
adoption; arbitrary `.o` or claimed `.checked` files are not verification
evidence. This does not add an untrusted proof-exchange protocol or expand
SCOPE-3 to trust arbitrary imported WF implementations.

Keep semantic values separate from implementation witnesses. A caller proof
refers to `(callee instance, clause identity, normalized relation)` and its
permitted availability, not a callee's local derivation node. The current build
maps that claim to the selected implementation's successful receipt. If the
implementation is rechecked and proves exactly that boundary, reuse the caller
derivation while binding the new witness. Changed meaning or availability
rechecks the consumer. Rebinding requires equality and dependency validation;
it cannot manufacture a different proposition.

The assembled program receipt identifies selected modules, interfaces,
instances and their checked components. All included bodies required by the
language must be checked, including unused nongeneric definitions and symbolic
generic templates. Checking only entry-reachable bodies would weaken present
acceptance. Required concrete instances are checked separately. No final
program or executable is published with a selected obligation pending or failed.

Atomic valid cache entries survive an unrelated failed build. A failed
prospective program has no lowering authority but does not erase unrelated
module work. Staged object production can run once its own proof dependencies
are valid; final publication waits for complete composition. An incompatible,
missing or incomplete cache entry causes recomputation, not source rejection.
Interrupted writes are not published.

Finalize and prune each local derivation fragment once when its producer
succeeds. Imported claims are explicit edges. Composition updates receipt
references and dependency validity without loading, replaying or remapping all
unchanged proof nodes. Source audit can materialize those nodes on demand with
current locations. DIAG-2's current whole-program reachability/remap operation
becomes local fragment finalization plus checked composition; retaining the
full traversal after every edit would defeat proof-level incrementality.

DIAG-2's single-program ownership/discard language therefore needs amendment:
fact constructors remain unchanged while evidence is retained in components
and assembled through validated references. There is no second solver or
proof-replay acceptance path.

## Recursive dependencies and generic instances

### Keep module, call and proof graphs distinct

Module imports may contain cycles. Requiring an acyclic source-module graph
would remove expressible architectures merely to simplify caching. Collect
participating declaration surfaces, then use the actual template graph for
FN-6 and concrete call graph for FN-9. A module cycle need not contain a call
cycle, and a proof component need not contain every body of those modules.
Header formation still detects illegal inline recursive types and visibility
errors under the existing rules; accepting an import cycle does not accept an
infinite layout or use-before-declaration inside a module.

Retain FN-9's rule: same-component postcondition summaries are unavailable
during checking, and each component publishes its clauses atomically. Imported
declarations are not exempt. Old cached summaries cannot let A and B prove each
other's postconditions after an edit makes them recursive.

Persist adjacency and reverse adjacency by stable node identity. An inserted
edge between components updates the condensation graph; when it closes a
cycle, merge the components in the connecting reachability region and
invalidate membership and summary-availability queries. On an internal edge
deletion, recompute SCCs inside the old component and propagate changed
boundary values. External edge deletion updates incident reachability
dependencies. Consumers always read current membership. A worst-case edit
can affect the entire graph; an unrelated body edit does not justify checking
every body.

Apply the same principle to FN-6's finite function-argument target closure and
the allocation closure used by no-heap and optimization. Deletion must retract
support: an add-only persistent fixed point is unsound across edits. Reset an
affected component's derived values to the specified initial state, recompute
using current predecessors, and compare the result. Every admitted automatic
family still runs to its specified completion without an acceptance budget.

For example, B's proof initially uses A's established postcondition. An A body
edit adds a call back to B without changing either signature. The new SCC makes
A's summary unavailable to B, which must be rechecked. Signature equality
alone is therefore insufficient: summary availability is a semantic dependency.

### Specialization is a shared build computation

Artifacts contain resolved templates and their symbolic checks. A demand for
`f<T, N, fn g>` requests an instance whose stable identity includes the defining
declaration and complete explicit argument vector. Its revision key also
includes the template revision and dependencies actually read. The executable
actual and authoritative formal contract stay distinct under FN-4. Private
helpers are available to the defining module's instance producer without
becoming accessible to client source.

A build-wide instance store coalesces identical requests. Instance identity,
rather than the consumer that requested it first, owns the emitted definition.
Templates and instantiated objects remain separate. Editing a generic body
rechecks affected instances; adding a consumer of an existing instance reuses
it. Parallel equivalent requests join one task rather than emit conflicting
definitions.

FN-6 checks the written template/function-binding graph before permitting
expansion. Newly selected bindings update that finite graph first. Concrete
checking, uninhabited-body judgments, postcondition availability and target
layout remain instance-specific. A symbolic check is not a blanket proof of
all future instances. Demand discovery closes deterministically over admitted
instance keys. Symbolic and concrete checking retain distinct context keys;
stable declaration IDs alone do not authorize promoting a symbolic scratch
analysis into a concrete proof. The current generic-validation-scope refusal
of that promotion continues to apply.

Nominal layout, release class and projections have separate keys. A private
representation edit can leave the source contract unchanged while affecting
callers that store, pass, release or optimize that nominal. Record all relevant
capability and representation consumers. Allocation ceilings and target
representability are checked at their actual sites; sharing a generic instance
does not erase per-call target obligations.

## Optimization without a source-module performance boundary

### Distinguish semantic and optimization dependencies

Source checking reads contracts, type properties, specified graph results and
local source, never optimizer decisions. Optimization additionally reads
bodies, layouts, profiles and program summaries. Preserve those different
edges throughout the dependency graph.

| Change | Checks that can remain cached | Required changes |
|---|---|---|
| Nongeneric body; same boundary and summary availability | Ordinary callers' typing, ownership and proofs | Changed body and objects using its old implementation |
| Consumed requirement, effect or postcondition | Unrelated consumers | Dependent call checks/proofs and derived objects |
| Private layout without changed semantic propositions | Layout-independent proofs | Layout, ABI, cleanup and optimized representation consumers |
| Generic template body | Unrelated templates and unchanged caller boundary reasoning | Affected instance checks and implementation consumers |
| Edge that merges proof SCCs | Components with unchanged availability | Merged components and changed summary consumers |
| Profile/inlining policy | Source formation and proofs | Affected optimization plans, objects and final link |
| Runtime/library/link selection | Source results with unchanged declared boundaries | Composition, relevant target checks and final link |

Inlining is a dependency even if the callee's public signature is unchanged.
Cost summaries and rejected import candidates also count: a large function
becoming small can enable new optimization without an old import edge.
Export, reachability and prevailing-definition changes can alter
internalization, propagation or elimination. An old import list alone is an
incomplete optimization key.

### LLVM fragments and optimization regions

Use independently addressable pre-optimization IR fragments for concrete
function families and generated release/parallel helpers. Module boundaries
neither prohibit import nor force every member into one LLVM module. Private
helpers can be imported and inlined through compiler metadata. Native runtime
and linked prelude bodies need equivalent optimization visibility where their
toolchain supports it.

The proposed LLVM integration uses ThinLTO-style summary analysis,
per-backend plans, body import and optimized-object caching. ThinLTO is a
starting mechanism, not evidence that every full-LTO opportunity or runtime
objective is already met. A measured missed specialization or constant
propagation requires a better summary/import plan, checked-IR transformation
before LLVM, or a joint optimization region spanning the relevant modules.
Do not close such a gap as an inevitable cost of modules.

Optimization regions may differ from fine-grained storage units. Compare one
function family per fragment against stable small groups; select grouping by
the validation criteria below. Group identity is derived deterministically
from stable members and policy, not ordinal position, cache history or task
completion order. A changed region can split/merge without globally renumbering
unrelated units. Initially use a concrete function plus its inseparable
local helpers as the independently cached bitcode unit; cross-module import
supplies optimization visibility. Larger stable groups are candidates to adopt
only when their measured construction benefit preserves runtime quality and
isolated-edit criteria, rather than an unspecified prerequisite to implementation. Bodies, generic instances and helpers remain independently
addressable even when jointly optimized. Cross-module recursion is allowed
inside an optimization region when the transformation benefits from it.

Function symbols use stable declaration/instance identity; implementation
revision belongs in the artifact key. Cross-unit declarations must agree on
calling conventions, layouts and target attributes. Emit one prevailing
definition per instance. Optimization-only copies must not become duplicate
native definitions. Constants, prelude bodies, release helpers, sequential
clones, recursive-budget variants and parallel thunks all have explicit owners;
a cross-unit reference cannot retain the old definition's `internal` linkage.

Existing parallel lowering computes clone reachability and recursive frontiers
from program metadata. Make those queries incremental and key each generated
function by its parent and actual policy inputs. Changed clone membership
legitimately invalidates generated bodies. Emit scheduler bootstrap, runtime
fallbacks and initialization once per linked program, not once per LLVM unit.
Direct self-tail lowering remains local and keeps its guarantee.

Preserve checked optimization facts only under a complete mapping to LLVM's
contract. No alias/capture/memory attribute follows merely from a WF effect
spelling. Imported bitcode carries the same justified facts as local bitcode;
removing optional metadata never changes acceptance. No abstraction boundary
adds a runtime bounds check, dictionary, lock or scheduling edge.

### LLVM facilities and required integration

LLVM provides distributed ThinLTO indexes, separate backend work, imports and
object caches [E3, E4]. Its cache includes imported module hashes, exports,
resolution facts and configuration [E5]. This is more conservative than hashing
only imported function text. Broad bitcode modules can cause avoidable
invalidation, so actual module granularity and cache behavior need measurement
on the selected LLVM release.

Stock ThinLTO does not make WF checking incremental, and its combined summary
analysis does not promise persistent incremental planning. Complete this design
with dependency-tracked optimization planning: persist function summaries,
graph components, resolution results and per-backend decisions. Recompute dirty
analyses from their specified initial states; materialize changed backend plans
and retain objects when the plan and imported content are unchanged.
Whole-program facts can legitimately invalidate many plans when their values
change. Rebuilding all optimizer analysis on every local edit is a comparison
implementation, not the endpoint.

The integration must expose each enabled analysis's complete read set and
its result. Global reachability, cost thresholds, exports, symbol resolution
and profile aggregates cannot become implicit inputs. An analysis with a
genuinely global dependency has a global query, while local component queries
retain independent results. Cache a global decision only when all its inputs
match; do not drop dependency edges to obtain attractive rebuild counts.
Changing a body while its optimizer summary stays equal avoids redoing
summary-only analysis, although importers of its body still regenerate.

Use LLVM-version-specific index/backend interfaces where sufficient. If
persistent planning needs additional state or instrumentation, a bounded
toolchain extension is required; it is not an existing `clang` flag.
LLVM remains behind the backend process boundary. WF's checker stays safe
Rust without unsafe FFI or another acceptance path. A stable optimizer artifact
protocol across LLVM versions is unnecessary.

### Runtime quality and profile use

The cold and warm optimized builds use the same optimization policy, CPU
features and profile snapshot. Incrementality must not lower optimization
quality. PGO is supported as an explicit tracked input, not a required runtime
measurement during checking. Changing it invalidates optimization consumers
while keeping source proofs.

Full LTO is a comparison, not a universal upper bound on performance. Require
causal investigation of every repeatable runtime regression, inspect missed
optimization remarks and resulting code, and retain the best supported
transformation for the workload. There is no mathematical guarantee of a
globally fastest equivalent executable or of constant rebuild work when a
widely imported implementation changes.

## Final native linking

Ordinary full native linking is permitted. The final link consumes current
cached or rebuilt native objects and runtime objects, together with an exact
composition receipt. Its inputs include the selected entry, target, libraries,
linker version/options, export set, layout/profile controls and platform
metadata. An unchanged complete input selection can reuse an already produced
executable; otherwise invoke the normal optimizing native link.

No incremental native linker, permanent padding, extra jump indirection or
reduced link optimization is required. Link time and output construction remain
visible in end-to-end measurements, so a fast compiler stage cannot conceal an
expensive final link. ThinLTO backend work is accounted separately from the
final native-object link even if a tool happens to run both inside one process.

Cache native runtime construction under its actual sources, headers,
preprocessor options, compiler and target. Avoid recompiling unchanged C/LLVM
runtime units on every WF invocation. Runtime objects must match their
declarations and current build selection; changing a link input cannot bypass
the existing SCOPE-3 obligations.

The native definition set must agree with checked composition. Object success
or matching symbol names alone is insufficient: mismatched nominal identity,
calling convention, target layout or implementation selection invalidates the
composition. Full linking does not repeat WF parsing, proofs, specialization
or LLVM optimization of unchanged units.

## Alternatives and selection grounds

| Alternative | Useful property | Disposition |
|---|---|---|
| Whole-program checking followed by LLVM splitting | Small backend change | Refused: every edit still repeats frontend/proof work |
| Whole-module timestamp/object cache | Simple scheduling | Refused: coarse invalidation; timestamps are not exact semantic inputs |
| Independent native objects with permanently opaque bodies | Easy code generation | Refused: source boundaries unnecessarily restrict specialization/inlining |
| Full LTO after every edit | Broad implementation visibility | Quality comparator, not the persistent incremental architecture |
| ThinLTO flag alone | Parallel backends and object cache | Insufficient: supplies neither WF proof reuse nor persistent WF optimization planning |
| Handwritten interfaces accepted as facts | Easy isolated checking | Refused: contracts require verified implementation evidence |
| Reject all module cycles | Easy scheduling | Refused: actual finite template/call rules can decide cyclic dependencies |
| Portable proof/object certificates and new verifier | Untrusted distribution | Not selected: version-private results under the existing compiler trust boundary serve this consumer |
| Fast unoptimized incremental path plus optimized full rebuild | Easy performance split | Refused: optimized incremental compilation is itself required |
| Dependency-tracked checking, specialization and optimization; ordinary final link | Independent reuse with implementation visibility | Proposed; correctness, cost and runtime quality require the evidence below |

The language root's existing rejection calls modules unverifiable fact-loss
surfaces. Verified interface/evidence composition addresses that premise:
callers use the declared boundary they already use, and composition requires
a currently verified implementation. Source privacy does not erase optimizer
visibility. This changes the ground of the old decision; it does not claim
that the old rule already permits modules.

The compiler's refusal of premature stable artifact/replay infrastructure
continues to apply to unrequested product protocols. The concrete new consumer
is reuse of checked work across invocations. Version-private results serve it
without a stable serialization ABI, second checker, crate split or remote
cache service.

## Required specification and implementation changes

This design PR does not edit the active specification. A future amendment
must update the affected rules together, not merely remove PROG-1's prohibition.

| Owner | Before | Proposed change |
|---|---|---|
| PROG-1/2/3 | One ordered bundle, no modules, unqualified entry | Closed selected module graph; ordered records within each module; qualified ordinary entry and composition |
| FORM-2/3, GRAM-1/2/3/4/5, DIAG-1 | One root and unqualified name roles | Canonical headers/exports, per-module trees, factored qualified roles and current module-aware coordinates |
| TYPE-6 | Whole-unit identity and constructor uniqueness | Qualified identity, local visibility, explicit imports/exports and field access |
| Type/ownership/release consumers | Descriptions in one inventory | Same judgments over imported descriptions; privacy grants no storage or release exemption |
| FN-2/4/6/9, ENT-3.S12 | Whole-unit instances and summary identities | Same instance and SCC rules across modules, with current cached claims and availability |
| DIAG-2 | One exact-program value owns/discards all evidence | Checked component fragments and assembled receipt; failed composition grants no authority, unrelated valid entries survive |
| STOR-6/8, EFF-3, PAR-1/2 | Whole-program target/allocation/parallel metadata | Same rules over complete tracked layout, allocation and call-summary dependencies |
| PRE-1 / native binding | Compiler-owned declarations and linked bodies | Bind selected prelude, runtime and target identity into composition/codegen inputs |

The candidate adds three fixed words (`module`, `import`, `pub`) and reuses
`::`. Exact META-5 production/rule deltas require the complete grammar and
judgment patch and its strong-LL(2) check. This architecture document does not
invent a count before that work.

| Current implementation owner | Required structural change |
|---|---|
| `source.rs`, syntax/canonical rendering | Module record sets, stable item identity, independent trees and source maps |
| `resolution/engine*` | Local inventories, persisted exports, explicit lookup and positive/negative dependencies |
| `semantic/check.rs`, `check/generics*` | Query-owned body/instance checking and reusable owned results instead of whole-unit borrow chains |
| `semantic/entailment*`, `postcondition.rs` | Stable claims, retained derivations, current SCC availability and composition |
| `semantic/model.rs`, allocation/permission consumers | Stable identities and tracked fixed-point/target dependencies |
| `lowering*`, `backend/emitter*` | Owned IR fragments, external declarations, helper ownership and reusable plans |
| `driver.rs`, `bin/whitefootc.rs` | Persistent scheduling, backend tasks, cached runtime objects and ordinary final linking |
| Diagnostics, stack/parallel ledgers, conformance adapter | Current locations, composition-wide reports and one cold/incremental checker entry |

Do not retain the old whole-program checker as a second semantic path.
Establish ordinary query responsibilities and dependencies, then persist
results. An empty cache executes the cold path through the same engine.
Implementation may land in reviewable slices, but a split-only backend or
object-only cache is not the completed capability.

Completion requires a real multi-module consumer exercising imported
contracts, an independently edited implementation, generic specialization,
a cross-module proof-cycle edit, optimized body import and ordinary final
linking through that same pipeline. Public artifact stability is unnecessary;
correct composition and complete dependency tracking are required.

## External evidence

Primary sources were inspected during 2026-09-21/22. They support mechanisms
and expose limits; they are not measurements of WF or proofs of this design.

- **E1 — [Rust incremental queries](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation-in-detail.html).**
  Explicit dependencies, result comparison and stable identities support
  stopping invalidation when semantic results stay equal. WF additionally
  needs its specified proof-availability conditions.
- **E2 — [Rust monomorphization and partitioning](https://rustc-dev-guide.rust-lang.org/backend/monomorph.html).**
  Instance collection and codegen partitioning separate source organization
  from generation. WF's build-wide ownership is a proposed design difference.
- **E3 — [LLVM ThinLTO](https://clang.llvm.org/docs/ThinLTO.html).**
  Summary analysis, body importing, parallel backends and object caches provide
  optimization across units. They do not establish runtime optimality.
- **E4 — [Distributed ThinLTO](https://www.llvm.org/docs/DTLTO.html).**
  Separate index/backend work offers per-backend integration points.
  Distribution itself is not required.
- **E5 — [LLVM LTO cache keys](https://llvm.org/doxygen/LTO_8cpp_source.html).**
  `computeLTOCacheKey` includes compiler/configuration inputs, imported module
  hashes, exports and resolution information. This motivates complete
  optimization dependencies and measuring fragment granularity.
- **E6 — [F* interfaces](https://fstar-lang.org/tutorial/book/part3/part3_interfaces.html).**
  Logical interfaces and implementing models illustrate modular verification.
  WF retains its own deterministic proof rules rather than importing another
  theorem language or solver.
- **E7 — [GHC separate compilation](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/separate_compilation.html).**
  Recompilation and inlined-definition dependencies show why direct source
  imports alone do not describe all implementation dependencies.

## Discriminating validation criteria

These criteria precede any experiment for selecting this design. No performance
measurements have been made for this proposal.

### Correctness and dependency precision

For a fixed specification, compiler, target, source snapshot, dependency
selection, and optimization policy, compare a fresh build with builds reached
through edits, reversions, process restarts, different worker counts, and cache
eviction. Require identical acceptance, rule attribution and current-source
diagnostic locations; accepted executables must have the same independently
checked results. Report diagnostic ordering separately where DIAG-1 permits
implementation choices.

Include private-body changes, contract changes, nominal-layout changes,
unused and newly visible declarations, generic body and argument changes,
function-kind bindings, new and deleted dependency edges, recursive-component
merges and splits, no-heap propagation, optimizer imports, and linker inputs.
Cold and incremental runs must agree even when a newly formed recursive
component removes a previously usable postcondition.

Count query execution and invalidation by phase. A repeated no-change build
executes no body checking, proof derivation, LLVM optimization, or object
generation. An isolated nongeneric body edit with unchanged semantic boundary
and recursion membership rechecks that body's verification unit; unchanged
callers reuse source proofs. Optimizer consumers that imported the old body
must be rebuilt. Invalidating every importer solely because a whole module's
source hash changed fails the precision criterion.

Cross-module privacy also needs executable/negative witnesses: private source
field selection rejects, exported accessor contracts transport their private
projection facts, and field construction/destructuring cannot bypass linear
release or existing opacity. Cache eviction and an unrelated failed module
never grant authority to an unverified component. Keep normative cold conformance
execution and targeted incremental edit sequences distinct; do not replace
conformance execution with shared cached test verdicts.

### Runtime quality and construction cost

Hold WF source, algorithms, target CPU, LLVM version, runtime, profiles and
observable results fixed. Compare the current monolithic backend, separate
objects without cross-module optimization, and the proposed optimized
incremental backend. A full-LTO build is a quality comparator, not an assumed
performance optimum. Attribute every repeatable loss to a concrete missed
optimization or layout decision; do not average away a regressing workload.

Measure cold construction, unchanged rebuild, implementation-only edit,
interface edit, generic edit, and broad dependency change. Separate WF parsing,
resolution, checking/proof, specialization, optimization planning, LLVM
backend, native linking, and executable publication. Record peak memory, CPU,
wall time, loaded bodies, rebuilt partitions, and cache size. Run same-image
controls and independent output oracles under the maintained performance
method. Use real multi-module consumers plus controlled scaling cases; neither
a tiny corpus nor synthetic scale establishes large-project runtime quality.

An optimization policy is selected only after these observations distinguish
it from the alternatives. Broad rebuilds caused by actual dependency changes
are reported as such; a small source edit is not a promise of constant work.

## Design suitability and remaining evidence

The selected architecture addresses the requested frontend and backend reuse
together. Its principal costs are complete dependency tracking, persistent
identity/evidence ownership, and LLVM planning integration. Those costs have
identified consumers: independent library edits, reusable generic instances
and optimized multi-module applications. A stable third-party artifact ABI,
remote cache service, new theorem language and incremental native linker do
not have a requirement here and are not selected.

The pending tree revision replaces the language root's closed-single-unit
decision, replaces name-resolution's global inventory with module inventories,
clarifies the compiler root's artifact boundary, and adds one compiler child
for persistent incremental computation. The constitution's safety and
performance priorities, fixed deterministic proof families, generic
specialization and finite-cycle rule, callable contracts, source ownership,
backend fact obligations, self-tail semantics and runtime model remain in
force. The current generic scratch-analysis rejection is retained: different
verification contexts cannot share proofs merely because IDs are stable.

Four uncertainties are implementation acceptance work, not weaker endpoints:

- Verify the complete qualified grammar, public/private construction rules,
  private-projection contract transport and exact specification deltas.
- Establish a compositional soundness argument for claim rebinding and
  component receipts, including cycle edits, deletion and failed-build cases;
  differential edit-sequence tests alone do not prove soundness.
- Qualify complete LLVM planning dependencies and choose fragment/optimization
  region granularity using measured runtime quality and edit costs.
- Demonstrate useful cold and warm costs on real module consumers. Start with
  the existing GrowVector library/caller separation, then modularize the
  existing wfgrep and SHA-256 programs without changing algorithms, and add a
  generic-heavy multi-module consumer plus controlled dependency-graph scaling.
  These are proposed consumers, not currently implemented module benchmarks.

The maintained TODO links these unresolved capabilities and criteria. This
design review can judge the architecture and its stated limits; it cannot
certify an unimplemented incremental checker or claim unmeasured performance.
