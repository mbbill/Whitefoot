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

Revisit: the structural acyclicity requirement remains, but coupling its
certificate to whole namespace subtrees is under reconsideration. A small
acyclic dependency change can otherwise require module moves, qualified-name
changes and edits across consumers. The ordered-tree argument below remains
valid; it does not establish acceptable change locality. An independently
declared module order is a comparison candidate, not a selected replacement.

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

### Selection criteria and alternatives

The handwritten module interface must itself expose the complete public
source contract. A client reads this module's `.wfm` and any explicitly
referenced dependency interfaces, without looking through its implementation
files or requiring a generated interface to discover missing declarations.
Self-containment is relative to explicit interface imports, not duplication
of every dependency's API into one file. Compiler inputs for layout,
specialization and optimization are a separate concern.

This requirement selects full public declarations over the earlier thin
export-list candidate. Checked repetition of a function declaration is an
acceptable cost of an independently readable contract; minimizing repeated
text does not meet the requirement by itself. Ordinary private implementation
edits must not change the public interface file. Module size still does not
select incremental checking or LLVM partition size.

| Candidate | Benefit | Cost and disposition |
|---|---|---|
| Per-source module/import headers and `pub` declarations | Definition and visibility appear together | Refused: the public contract is spread across implementation files |
| Thin export list plus generated complete interface | Each signature and contract is written once | Superseded: the handwritten file alone does not satisfy public-interface self-containment |
| Complete public declarations in one `.wfm`, with checked ordinary implementations | A caller or agent can read and hold the written contract fixed independently of implementation | Selected; the compiler must enforce declaration correspondence and the implementation must prove the declared obligations |
| Complete interface with body-only implementation bindings | Avoids repeated function headers | Not selected: ordinary complete definitions remain locally readable and avoid a second body-binding form; repeated declarations are mechanically checked |
| Canonical module directory with direct implementation-file membership | Filesystem and module ownership have one spelling | Selected under the path-based namespace requirement; snapshot and track the directory inventory, while complete .wfm declarations still exclusively control publication |
| Recursive directory collection or automatic export | Few written entries | Refused: recursively collecting a child implementation confuses ownership, and adding a private definition must not publish it |

OCaml's checked interface/implementation pair [E11] supports this mechanism;
it does not establish WF's proof composition. Clang module maps [E8], Java
module declarations [E9] and Haskell export lists [E10] remain useful
comparators, but their existence does not make a name list a complete WF API.
No collaboration or performance trial has been run. Before such a trial,
hold APIs, algorithms and proof obligations fixed and record interface-reading
errors, declaration-mismatch diagnostics, concurrent edits and coordination.
These observations may improve tooling or layout; they do not override the
selected requirement that the handwritten public contract stand on its own.

### One complete public interface file

Each source module has exactly one `.wfm`. Its root-relative path owns the
module name; its contents own public interface imports and complete public
declarations. Every ordinary top-level
declaration in that file belongs to the public API; there is no separate
export list or implementation-side `pub` switch. Public function declarations
include generic parameters and bounds, parameter/result labels and modes,
types, capabilities, effects, and complete `requires`/`ensures` clauses.
Public constants and interface/binding groups include their public definitions.
Public concrete records and enums declare their externally visible schema
there. No interface fragment, textual include, wildcard export or forwarding
alias can fill in an omitted part from an implementation file.

The build explicitly selects canonical source roots and dependency identities.
The filesystem rule below determines module paths and direct implementation
membership within that selected source snapshot; there is no separately
editable module-name or member-file map. Implementation-only dependencies
remain explicit private build inputs, since adding one should not rewrite the
public interface. Their configuration spelling is not selected here; they
name exact module paths under selected roots, not arbitrary file aliases.

An interface import identifies an external module required to interpret that
interface, never one of its own source files. Each root resolves to one build-
selected dependency identity. Implementation-only dependencies are visible to
implementation records, but cannot resolve an interface declaration. Public
and private dependency bindings for the same root must agree; the build cannot
inject public declarations or override interface imports. Version acquisition
and package solving remain outside the language.

Illustrative contents of `counters.wfm`, not an accepted grammar. The path
already declares the module, so no second written module name is needed:

```text
fn advance(value: own u64) -> next: own u64 pure contract {
  requires value < 18446744073709551615_u64;
  ensures next == value + 1_u64;
};
```

An ordinary implementation in one selected `.wf` repeats that function's
declaration and supplies its body. Other selected files may define private
helpers without mentioning them in `.wfm`. There are no per-file module,
import, export or namespace declarations.

Interface formation can check names, types and contract well-formedness
without implementation bodies. That does not prove an implementation exists
or satisfies the interface. Lowering and accepted composition still require
current checked implementations and the existing recursive summary rules.
An interface file is neither a theorem assumption nor an object-code promise.

### Checked interface and implementation correspondence

Resolve the public declarations first, then collect the complete local
implementation inventory. A public function definition binds to the
interface's stable identity; it does not introduce a second function or
overload. Exactly one selected definition must implement each required public
callable, subject to the existing compiler-owned native binding rules. Missing,
duplicate and mismatched implementations reject at the related declarations.
An unlisted ordinary function is private even when another source file uses it.

Require equality of the normalized declaration: generic arity/kinds and bounds,
named-call labels, parameter/result modes and types, capabilities, effect row
and contract clauses. Normalization follows the specified deterministic rules;
it may identify permitted bound-variable renamings while retaining every
caller-visible name. It is not arbitrary logical equivalence or a solver for
whether one contract refines another. The body is checked against this same
declaration. A stronger local fact does not silently enlarge the public
postcondition; a different public contract requires an explicit interface edit.
Internal calls to a public function use the matched contract and normal
summary availability, not a second private strengthening.

Public concrete type, constant and group definitions in `.wfm` are directly
available to implementations. They need not be redeclared in every `.wf`,
and must not acquire a fresh nominal identity there. Representation-hiding
nominal interfaces require an explicit public capability description and a
checked match to the implementation representation. The exact nominal
correspondence judgment and its logical vocabulary remain required language
design work; the existing `opaque` modifier is not repurposed, since it
already restricts construction even in its declaring scope.

### One module namespace and declaration formation

All implementation files form one module namespace and one local declaration
inventory. A helper defined in one file can be used by another, including
mutual function recursion, without an interface entry, import or export.
File boundaries are locations for authorship and diagnostics, not privacy,
name-resolution or source linking boundaries. The public interface selects
which of these identities external modules may use; it does not mediate
internal references.

Each module's own implementation has a flat local inventory. Directory paths
organize modules into qualified namespaces as specified below; a child `.wfm`
introduces another module, not another file in the parent's private scope.
No separate `namespace` block is introduced. Internal code sees its own
module-private declarations, and local spelling collisions still need ordinary
diagnostics. Splitting a file inside the same module does not grant privacy.

Local uses resolve against the full inventory; dependency uses have an
explicit root-qualified path such as `app::counters::advance`. Module roots and
local
names have unambiguous ownership. There is no wildcard import, implicit
transitive import, overload search or cross-module namespace extension.
Selected source/dependency identity, canonical module path and local declaration
identity determine a nominal, not its printed path without the selected root.
Implementation filenames do not determine declaration identities. Re-exports
and export renaming remain unselected; a stable facade over independently changing
modules is the concrete consumer that would reopen that choice.

Collect all top-level names before resolving definitions. Functions, nominals,
constants and interface/binding groups are visible independently of file/item
order. This changes TYPE-6's lexical visibility of non-function top-level
declarations and CONST-2's earlier-declaration requirement. Name availability
does not establish a valid value, type or proof: constant dependencies and
group expansion remain acyclic, and finite instantiation and finite layout
retain their actual rules, including admitted recursion through indirection.
Parameters, fields, generic binders, contract definitions and local bindings
keep their owner-local/lexical scope. Existing reserved-name, collision and
no-shadowing rules remain; new globals can invalidate lookup or scope checks.

Qualification must reach types, constructors, callees, function arguments,
binding groups, constants, match variants, projections, destructuring and
postcondition routes together. Fixed operation-table names and numeric bounds
do not gain dependency lookup. Existing `::` generic-call syntax requires
factoring with qualified names in the strong-LL(2) grammar; this document does
not claim that the complete productions have been checked.

The build selects a module-qualified entry. The selected root interface owns
`program no_heap;`, which constrains the entire selected dependency closure,
including implementation-only dependencies, concrete instances and prelude
definitions. Moving an allocation into a private dependency cannot hide it.

### Filesystem namespace paths and module ownership

Bind a canonical source root to one explicit root name, such as `app`. Within
that root, `vector.wfm` declares `app::vector`, and `vector/other.wfm`
declares `app::vector::other`. Qualified module paths mirror directories and
the interface basename; there is no independent module-name declaration or
directory-to-namespace remapping. An external dependency root also identifies
the selected dependency instance, so equal relative paths in different roots
do not identify the same nominal.

```text
vector.wfm
vector/
  core.wf
  growth.wf
  other.wfm
  other/
    core.wf
```

The direct `.wf` records in `vector/` implement `app::vector`. The direct
`.wf` records in `vector/other/` implement `app::vector::other`. Neither the
child interface nor its implementation is part of the parent implementation.
Do not recursively collect `**/*.wf`. A selected implementation record must
have the matching interface for its immediate owning directory; an unmatched
record cannot silently inherit some distant ancestor's ownership. A directory
prefix such as `a/b/` may organize `a/b/c.wfm` without `a.wfm` or `a/b.wfm`:
such prefixes name locations, not implicit modules or dependency graph nodes.

This replaces explicit member-file lists for the selected layout. Input
formation captures a closed source snapshot and inventories direct entries
deterministically; changes to membership are tracked inputs. Directory entry
order, working directory and an ambient search path cannot choose meaning.
Adding a private file changes the implementation inventory but cannot add a
public declaration, since `.wfm` remains the complete publication authority.
The earlier objection to discovery coupled with automatic publication does
not justify maintaining a second ownership map under this requirement.

Filesystem path uniqueness is only part of naming correctness. The compiler
still rejects duplicate declarations across a module's implementation files,
conflicting root bindings and ambiguous canonical source paths. Reserve a
child namespace component against a top-level declaration with the same name
in its parent module: `vector/other.wfm` can coexist on disk with an exported
function called `other` in `vector.wfm`, but that conflicting namespace is not
accepted. The namespace inventory records those path components and negative
lookups; it does not infer access privileges from their existence. Canonical
path/case/alias rules must yield the same names on supported hosts, with
ambiguity diagnosed rather than resolved by filesystem iteration order. Their
complete acceptance spelling remains part of grammar/input qualification.

Moving a function between direct files of the same module preserves semantic
identity. Moving or renaming a module path changes its qualified identity and
requires affected imports and uses to change. Moving only the bound physical
root while retaining its selected source identity and relative paths does not
rename every module. Path-derived naming deliberately trades free module
relocation for one inspectable relationship between source layout and names.

### Structural dependency order

The required property is stronger than rejecting a cyclic set of imports:
the declared architecture must make every permitted set of module edges
acyclic. An added dependency should have a locally decidable architectural
direction, without searching an existing import graph for a cycle or choosing
which edge of that cycle to remove. The previous arbitrary-edge DAG proposal
does not meet that requirement and is superseded.

Select a structural candidate by these criteria: legality uses only the two
module paths and declared architecture metadata; every permitted edge strictly
descends one finite order; public and private dependencies follow the same
rule; shared services can have several consumers; and an illegal direction is
reported against an existing architectural boundary. Also assess how much
source and architecture metadata must change when dependencies evolve without
changing module APIs. These requirements do not imply a uniquely best grouping
of application responsibilities.

| Candidate | Structural guarantee | Cost and disposition |
|---|---|---|
| Imports only to strict descendants | Increasing finite tree depth prevents cycles | Too restrictive for shared services across branches without another mechanism |
| Imports only to greater directory depth anywhere | Depth alone certifies acyclicity | Rejects equal-depth sharing and makes dependency levels depend on incidental nesting |
| An explicit rank for every module, with imports only to lower ranks | Each edge can be checked without graph analysis | Revisit alongside a centralized sequence of canonical module paths: preserves names when order changes, but rank propagation or shared-order editing can still require coordination |
| Ordered sibling subtrees, with each parent after its descendants | Paths and local sibling orders certify acyclicity | Revisit: direction is explicit, but some acyclic edits force regrouping and path changes; the proof does not establish practical evolution costs |
| Arbitrary imports followed by cycle detection | Detects a cycle after composition | Superseded: no structural certificate or prior direction tells an author which new dependency is disallowed |

### Ordered namespace tree: candidate under reconsideration

Annotate each branching namespace with an explicit order of its immediate
child namespace components, from dependency providers to consumers. This is
architectural input in the existing build description, separate from complete
public `.wfm` declarations. It adds order to the filesystem-derived tree;
it does not rename paths or override file ownership. Exact configuration
syntax remains to be specified. For example, the following is mathematical
notation for two local lists, not accepted WF or build grammar:

```text
children(root)   = [shared, d, a]
children(shared) = [memory, f]
```

Every branching node lists each existing immediate namespace child once, with
no duplicate or unknown entries. Every selected module identity has exactly
one canonical position; aliases cannot give it different positions. Zero or
one child needs no choice of order.
The finite namespace inventory includes module paths and their directory
prefixes; prefixes without `.wfm` can therefore own child-order metadata
without becoming modules. Multiple selected source roots sit under one
virtual root with its own declared order. An out-of-tree dependency is not an
exemption: first bind its root and include it in this architecture. Order is
never inferred from imports, alphabetical names or directory enumeration.

A module is conceptually after all modules in its own descendant subtrees.
For a declared dependency `A -> B`, apply exactly these local cases:

1. If B is a strict descendant of A, allow it.
2. If B is A or a strict ancestor of A, reject it.
3. Otherwise find their lowest common namespace ancestor. Allow the edge
   exactly when B's immediate branch there precedes A's immediate branch.

These rules apply to public interface imports and private implementation
dependencies alike. No source reference gains an exemption through re-export,
a path alias, a native binding or an implementation-only declaration.
Interface access still requires the direct dependency and B's public API;
there is no automatic visibility of earlier branches, transitive dependency
or familial private access.

A parent may therefore directly depend on a child or grandchild. A descendant
cannot depend on its ancestor's module interface, even when that isolated
edge would be acyclic. Cross-branch imports may target any depth of an earlier
branch; no intermediate interface forwarding is required. A naming prefix
without `.wfm` cannot itself be an import destination.

### Why local checks guarantee a DAG

For the proof, traverse the finite ordered namespace tree in postorder: visit
each child subtree in its declared order, then the node's own module if one
exists. Give each visited module a distinct position `rank(M)`. The compiler
does not need to materialize these global numbers to check an import.

In case 1, a descendant B is visited before its ancestor A. In case 3, all of
B's branch is visited before any module of A's branch. Thus every allowed
`A -> B` satisfies `rank(B) < rank(A)`. A directed cycle would require a
strictly decreasing sequence of finite positions to return to its starting
position, which is impossible. Rejecting self and ancestor edges covers the
remaining cases. Restricting the tree to any selected module subset preserves
this argument.

Only namespace-tree consistency and per-dependency direction checks are
required to establish source-module acyclicity; no import-graph SCC, cycle
search or inferred topological sorting is needed for that property. Reading
each declared dependency is still necessary, and incremental checking still
records actual consumed dependencies. Concrete generic/callback calls and
proof publication retain their separate FN-6/FN-9 graph checks; this theorem
is about source module dependencies, not every compiler or runtime graph.

As a supporting exploratory check, enumerate all ordered rooted tree shapes
with 1 through 9 nodes using preorder child-count sequences. Start with one
open node slot; consuming a node with k children changes slots to slots-1+k,
never exhaust slots before the final node, and finish at zero. Build each
tree from that sequence, independently assign postorder positions, and compare
the three-case path predicate against `rank(target) < rank(source)` for every
ordered pair including self-pairs. All 2,056 trees and 151,719 pairs agreed;
67,071 pairs were allowed. The tree counts by size were
`1, 1, 2, 5, 14, 42, 132, 429, 1430`. This finite model is not a compiler
test or the proof of the general theorem; it checks the candidate's case
definition against the argument. No performance conclusion follows from it.

### Shared services and directed architectural changes

With `children(root) = [shared, d, a]`, the original
`a/b/c -> d/e/f` example is legal: its target lies in the earlier d branch.
The reverse cross-branch direction is forbidden by the declared architecture,
without inspecting either branch's existing imports. Both subtrees may use
`shared/f`. Merely having a root order does not add any of those edges.

If d and a each need part of the other's behavior, the existing order identifies
the offending direction: d cannot acquire an a dependency. The adjustment is
to place genuinely shared contracts or implementation below both consumers,
for example in the earlier shared branch, while keeping d-specific and
a-specific adapters in their own branches. Existing type/constant/function-kind
parameters may express differing policies under their ordinary WF rules.
A lower component can receive behavior through such an explicit contract
without importing its higher-level caller; concrete call cycles still undergo
ordinary proof checks.

A shared f must itself obey its position. It cannot become shared merely by
moving its files while retaining dependencies on d or a. Extract its neutral
part and leave higher-level adapters behind, move a genuinely cohesive set
to the earlier branch, or revise the declared architecture deliberately.
Unrelated or incompatible requirements may need distinct implementations
instead of a common abstraction.

This gives a rule for which direction is disallowed, not an automatic choice
of the correct business abstraction. Changing sibling order is an explicit
architecture change: revalidate affected declared edges, rather than accepting
a new order just because one new import requests it. Where two parts truly
need unrestricted static mutual access, one WF module with several files
remains the existing organizational option. The policy does not suppress
ordinary recursion inside that module.

The additional subtree constraint is real. Consider two branches A and B
with independent modules and only these edges:

```text
A/a1 -> B/b1
B/b2 -> A/a2
```

This module graph is acyclic, yet neither ordering of the two intact branches
admits both edges. The tree policy requires separating lower shared providers
from their consumers or regrouping the branches. If preserving such
interleaving inside fixed directory groups becomes a requirement, the explicit
per-module rank alternative is preferable; it retains local acyclicity checks
without imposing one direction on entire subtrees. Do not hide this difference
by claiming the ordered-tree model represents every DAG under every layout.

### Dependency changes and source edit locality

Distinguish three cases. Adding an already permitted edge changes its explicit
dependency declaration and actual uses. Changing a child order can remain one
architecture edit if all existing edges still pass. A change that requires
interleaved directions across intact subtrees cannot be repaired by either
sibling order; regrouping then changes path-derived identities and may require
many imports, qualified uses and public references to be edited. Only the last
case necessarily exposes the naming cost of that repair; do not describe every
dependency edit as a repository-wide move.

Compare an explicit sequence of canonical module paths in the existing build
description, independent of directory grouping. An import may target only an
earlier entry. Unique complete membership and strict position descent provide
the same kind of local acyclicity certificate, while changing the sequence
does not rename modules or rewrite unchanged qualified references. For the
two-branch example, `[A/a2, B/b1, B/b2, A/a1]` admits both edges. This candidate
would apply one position rule to parent/child pairs too; it must not also grant
unconditional parent-to-descendant permission that bypasses that rule.

This does not promise constant-size repairs. Moving an entry can invalidate
existing edges and require several entries to move. A genuinely cyclic new
dependency set has no valid sequence. A centralized order reduces the number
of files holding the certificate, but becomes shared architectural input for
concurrent agents; numeric ranks distributed among modules avoid that single
file while risking multi-module relabeling. Neither representation can be
claimed to solve edit locality merely because its DAG proof is simple.

Under a fixed certificate and an independent per-edge predicate, both opposing
directions cannot be permitted: each edge alone is acyclic, but permitting
both would admit their cycle. Consequently some acyclic additions must change
the certificate or be refused. This is not a proof that many source files must
change; certificate maintenance and source-name changes are different costs.

The owner's change-locality concern reopens the ordered-tree recommendation.
Keep the structural guarantee and path-derived names; compare where the
dependency certificate lives before selecting their coupling. No replacement
policy or automatic reordering tool is selected in this discussion refinement.

Parents and children continue to share no private access. A separately
declared child interface is directly usable when the structural direction
allows it, not private merely because it is nested. Java's package hierarchy
[E9] is a precedent only for separating naming from private access, not evidence
for this new structural order. A parent-private separately compiled subtree
remains an unselected visibility capability with its own reopening consumer.

### Public semantic closure, representation and proof paths

A public declaration must be understandable using its interface, explicit
dependency interfaces and the ordinary language/prelude rules. Its type,
effect and proof expressions cannot name an implementation-only helper,
constant, group or private representation path. This is stronger than making
a generated interface carry private field identities behind a reader's back.
Private implementation contracts may still use private projections normally.

Public concrete records expose their declared schema; construction,
destructuring, `readonly`, ownership and release retain their ordinary rules.
Public enums expose all variants and payloads needed for exhaustive matching.
An abstract public nominal can hide representation only after its caller-
visible capabilities and any usable logical observations have been declared
and checked against that representation. Declaring an abstract name alone
does not grant layout, copying, dropping or proof capabilities.

The earlier independent review identified a contract-composition gap: under
FN-8, a client cannot call an ordinary accessor inside `requires`/`ensures`,
and source privacy prevents restating a private-field condition. Complete
public declarations make this gap explicit; copying the old private path into
`.wfm` would not satisfy semantic closure. The current GrowVector contracts
therefore still need a public representation or a separately designed checked
logical vocabulary before serving as an abstract module API. Public proof
projections/contract abbreviations remain an unresolved owner-direction
question. This revision does not select a remedy or claim the P2 is closed.

Self-containment concerns the source API, not all compiler information.
Compiler-owned artifacts retain checked implementation evidence, generic
bodies and private layout/release descriptions for specialization, by-value
representation and optimization. A client author need not read those bodies.
Source privacy creates no mandatory boxing, dynamic dispatch, runtime checks
or destructor escape. A layout change may invalidate code generation even
when the written interface is unchanged; dependency interface changes can
also alter resolved API meaning without editing this module's `.wfm`.
Composition validates those actual identities and inputs.

### Public interface and generated compiler projections

The handwritten interface is the public contract's authority. Resolved
interface data, checked implementations and the selected build inputs
generate distinct projections:

| Projection | Content | Readers |
|---|---|---|
| Name surface | Public declaration identities, kinds and signatures | External lookup and declaration formation |
| Semantic boundary | Modes/types, capabilities, effects, normalized contracts, constants and bounds | Source checking and proofs |
| Verification evidence | Matched interface, checked implementation, obligations, derivations and proof dependencies | Composition and audit |
| Generic template | Resolved implementation body, symbolic check and explicit parameters | Concrete instantiation |
| Physical description | Target layout, ABI, release shape and target obligations | Lowering and code generation |
| Optimization description | Checked IR, call edges, justified facts and cost/profile summaries | Planning and body import |

An internal inventory additionally contains private definitions and their
contracts. Public and internal views reference the same declaration identities
rather than serializing an interface and reading it back to call a local helper.
One compiler-private artifact may contain these lazily loaded sections; these
are not six public formats or a promise of cache compatibility across versions.

A generated interface renderer/comparison remains useful for resolved dependency
identities, normalized contracts and ABI differences. It supplements the
complete handwritten API rather than supplying its missing clauses. Compare
the same metadata that incremental checking consumes; use conservative
structural comparison, not a semantic equivalence oracle or checked-in API
lockfile. Changing only an implementation header to strengthen a requirement
now fails correspondence instead of silently changing the exported API.

### Module collaboration and incremental ownership

An implementation task can hold the public `.wfm` fixed while changing several
files in the module's shared namespace. Public signature changes edit both
interface and corresponding function definition; private helper/file changes
do not edit the public interface. Direct directory membership is a tracked
source input; implementation dependency changes still touch private build
configuration, where concurrent edits can conflict. Neither duplication checks
nor a flat namespace proves improved
multi-agent throughput. Cohesive contracts and independent changes remain
better module boundaries than a fixed file/line count or an exclusive agent lock.

Cache interface declarations, implementation declarations, correspondence,
body checks and lookup results separately. Adding a private file checks its
definitions and affected lookup/scope consumers, not all module bodies.
Reordering the canonical source inventory changes no semantic identity. Moving
a definition between files in the same module preserves its semantic identity and proof
dependencies; source maps and debug-information consumers may still change.
Never key every body or object by the entire `.wfm` or build-file digest.

Validate each import against its actual path relationship and relevant child
order. Inserting another sibling may change conceptual postorder numbers but
need not change existing legality results or declaration identity. Changing
an order revalidates affected edges; unchanged semantic claims retain their
ordinary reuse conditions. Do not put a global rank or whole architecture-file
digest into every body-check key. The architecture constrains permitted edges;
it neither creates all permitted dependencies nor forces serial compilation
of modules that do not actually depend on one another.

## Compilation units and file boundaries

One source module is one logical compilation unit: it has one public interface,
one private declaration inventory, and a checked composition of all selected
definitions. Several implementation files are inputs to that unit. No file
needs its own exported header, object identity, source import or native link
step just to call a helper in another file.

This does not choose the units of compiler computation:

| Boundary | Selected responsibility |
|---|---|
| Source file | Storage, editing, parsing and source coordinates |
| WF module | Public API, shared internal namespace, privacy and implementation ownership |
| Checking query / proof component | Reuse of declarations, correspondence, body judgments and recursive proof publication |
| LLVM optimization region | Bodies that need joint optimization under the chosen policy |
| Cached backend unit / native object | Machine-code reuse and final native-link input |

Merging the parsed declarations into one inventory is name resolution, not
linking separate source files. The compiler can cache a helper's checked body
while resolving all files against that inventory. Repeated declarations in the
public interface and definition are compared once per affected identity;
private helpers never need public interface entries. This is compatible with
fine-grained parsing, checking, proof and lowered-IR reuse.

An indivisible LLVM module per WF module is a different, viable backend choice.
It gives LLVM direct visibility of all local bodies and usually produces one
native object without internal backend partition references. Unchanged modules
can reuse that result; a changed module can still reuse WF frontend queries.
With the ordinary LLVM pipeline, however, changed bitcode generally causes
that backend unit's optimization and object generation to run again. Retaining
parsed ASTs or unoptimized function IR does not make those LLVM stages
incremental. One object per module also does not remove final cross-module
linking or the need for cross-module optimization.

The selected endpoint therefore makes the WF module a logical unit without
requiring one permanent LLVM module or object. Compiler-owned backend units
may contain functions from several source files, and an optimization region
may cross WF modules. No source filename determines that partition. Native
references between different backend objects are resolved by the permitted
final link; this proposal does not pretend those references disappear. They
require no writer-facing file interfaces or stable file ABI, and do not by
themselves require a dynamic call or runtime indirection. Inlining availability,
code layout and construction costs still depend on the actual optimization
plan and must be measured.

This distinction is supported by the separation of source organization and
codegen partitioning in E2, and summary/import/backend stages in E3. Neither
source proves the selected WF policy fastest. A single LLVM unit remains a
comparison and may be a measured joint region where useful; fixing every WF
module to that indivisible unit is not selected, since it limits backend reuse
without being necessary for the requested internal source semantics. If the
requirement becomes literally no inter-object references inside a WF module,
state the resulting whole-unit LLVM rebuild cost explicitly, or demonstrate
a different incremental LLVM backend; do not claim both from a cache flag.

## Persistent computation and identity

### Query boundaries

Use deterministic dependency-recording queries with explicit input values.
These families name responsibilities, not a proposed public Rust API:

| Query | Relevant inputs | Reusable output |
|---|---|---|
| Source formation | Interface/source bytes, selected canonical roots, direct directory inventory, grammar/spec identity | Tokens, canonical trees, source maps |
| Module surface / lookup | Canonical path components, public/private inventories, direct dependency paths, lookup role and spelling | Stable declaration or diagnostic |
| Module dependency permission | Canonical source/target paths and consumed architecture-certificate relations; descendant/divergent-branch order in the tree candidate | Allowed direct dependency or architectural direction diagnostic |
| Interface correspondence | Resolved public declaration and selected implementation declaration | Matching identity and checked normalized declaration, or diagnostic |
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
module identity, declaration domain and name; item-local node identities plus
the current source map recover diagnostic locations. Concrete instances add the complete
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

An explicitly declared descending order certifies source-module acyclicity by local checks
on both interface imports and implementation dependencies. Its declared order
identifies the forbidden direction before a new edge can form a cycle. Shared
lower contracts, ordinary function-kind parameters and a combined module are
the organizational choices described above; their actual suitability depends
on the API. No such choice requires dynamic dispatch, whole-module checking
or a whole-module LLVM unit.

This revises the earlier candidate permitting arbitrary module cycles. That
candidate is technically compatible with incremental compilation; a cycle
does not imply rebuilding all its bodies. It needs joint interface formation,
and preserves distinct module privacy boundaries within the cycle. The new
requirement instead makes permitted source dependencies acyclic by structure.
Its cost is a constraint on decomposition: combining
modules also combines their module-private access domain, and extracting a
common interface may change the public API. Reopen this choice if a concrete
consumer needs distinct cyclic privacy/distribution boundaries that a shared
module and ordinary interfaces cannot preserve satisfactorily; doing so would
reopen the structural source-DAG requirement itself. An acyclic but interleaved
directory decomposition is part of the reopened comparison with independent
module orders while preserving that requirement.

An acyclic module graph does not make actual calls acyclic. Functions inside
a module may recurse, and binding a function-kind parameter can create
cross-module concrete call cycles without a reverse source import. Use the
actual template/binding graph for FN-6 and the concrete call graph for FN-9;
a proof component need not coincide with a module. Header
formation still checks constant/group dependency cycles, finite nominal
instantiation and finite layout. Name availability is not evidence for those
judgments.

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
| One indivisible LLVM module/object per WF module | Direct local optimization visibility and no internal backend-object references | Viable comparison, not a language guarantee or the selected incremental endpoint; frontend reuse survives but LLVM work is generally module-grained |
| Independent native objects with permanently opaque bodies | Easy code generation | Refused: source boundaries unnecessarily restrict specialization/inlining |
| Full LTO after every edit | Broad implementation visibility | Quality comparator, not the persistent incremental architecture |
| ThinLTO flag alone | Parallel backends and object cache | Insufficient: supplies neither WF proof reuse nor persistent WF optimization planning |
| Handwritten interfaces accepted as facts | Easy isolated checking | Refused: contracts require verified implementation evidence |
| Cyclic module dependencies with joint interface formation | Retains separate module boundaries in a cyclic decomposition | Technically viable, but contradicts the structural source-DAG requirement; reopening needs a concrete cyclic-boundary consumer |
| Only adjacent or descendant directory imports | Tree alone certifies acyclicity | Too restrictive for shared providers across branches; ordered sibling subtrees add a checked sharing direction |
| Arbitrary cross-subtree imports plus cycle detection | Permits any acyclic grouping | Superseded: the required architecture must certify every permitted edge set before graph analysis |
| Explicit rank per module or central module sequence | Local strict-decrease proof without requiring subtree direction | Revisit for change locality: names can remain stable, but distributed relabeling or shared-order contention remain costs to evaluate |
| Direct deep imports plus automatic transitive source visibility | Short import lists | Refused: changing an intermediate module's dependencies would silently change a client's lookup surface |
| Parent or child inherits private access | Convenient family implementation | Not selected: separately declared modules use one uniform interface rule; keep private cooperating files in one module, and reopen family visibility only for a concrete separate-module consumer |
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
| PROG-1/2/3 | One ordered bundle, no modules, unqualified entry | Explicit finite architecture order certifying public/private source dependency acyclicity by local direction checks; revisit subtree coupling versus independent module order; path-derived modules, direct .wf ownership and checked composition |
| FORM-2/3, GRAM-1/2/3/4/5, DIAG-1 | One root and unqualified name roles | Separate complete interface/source forms, module-qualified names and diagnostics joining declarations and definitions |
| TYPE-6, CONST-2, FN-3 | Whole-unit identity; non-function top-level visibility follows source order | Path-qualified modules with shared local names; only structurally permitted direct interfaces are visible; ordinary privacy, order-independent top-level names, dependency validity and lexical local scope retained |
| Public declaration correspondence / type representation | No separate interface or public/private source boundary | Self-contained public semantic declarations, exact normalized callable correspondence, one nominal identity, and checked abstract representation/capability correspondence |
| Type/ownership/release consumers | Descriptions in one inventory | Same judgments over imported descriptions; privacy grants no storage or release exemption |
| FN-2/4/6/9, ENT-3.S12 | Whole-unit instances and summary identities | Same instance and SCC rules across modules, with current cached claims and availability |
| DIAG-2 | One exact-program value owns/discards all evidence | Checked component fragments and assembled receipt; failed composition grants no authority, unrelated valid entries survive |
| STOR-6/8, EFF-3, PAR-1/2 | Whole-program target/allocation/parallel metadata | Same rules over complete tracked layout, allocation and call-summary dependencies |
| PRE-1 / native binding | Compiler-owned declarations and linked bodies | Bind selected prelude, runtime and target identity into composition/codegen inputs |

The candidate uses the .wfm path as the module declaration, places direct
interface imports and complete public declarations in that file, omits a
second written module name, implementation-side `pub` and namespace blocks,
and reuses `::` for qualified names. Direct directory membership determines
implementation records; roots and private dependency bindings are explicit
build inputs together with an explicit dependency-order certificate whose
subtree or independent-module representation is under reconsideration. Exact declaration
terminators, canonical path/collision and order-consistency rules,
abstract nominal/capability syntax, normalized correspondence and dependency
selection syntax remain to be specified. META-5 deltas require the complete
grammars and judgments with strong-LL(2) checks; no count is invented here.

| Current implementation owner | Required structural change |
|---|---|
| `source.rs`, syntax/canonical rendering | Canonical selected roots, direct directory snapshots, interface/source grammar roots, stable path/item identity and source maps |
| `resolution/engine*` | Ordered path namespaces, local dependency permission checks, exact direct interfaces, shared local inventories, public closure, correspondence and positive/negative lookup dependencies |
| `semantic/check.rs`, `check/generics*` | Query-owned body/instance checking and reusable owned results instead of whole-unit borrow chains |
| `semantic/entailment*`, `postcondition.rs` | Stable claims, retained derivations, current SCC availability and composition |
| `semantic/model.rs`, allocation/permission consumers | Stable identities and tracked fixed-point/target dependencies |
| `lowering*`, `backend/emitter*` | Owned IR fragments, external declarations, helper ownership and reusable plans |
| `driver.rs`, `bin/whitefootc.rs` | Interface/build selection, supplementary interface comparison, persistent scheduling, file-independent backend tasks, cached runtime objects and ordinary final linking |
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
- **E8 — [Clang module maps](https://clang.llvm.org/docs/Modules.html#module-maps).**
  A separate record can map source headers to a logical module. WF does not
  inherit textual inclusion, umbrella discovery, configuration macros or
  wildcard forwarding from that system.
- **E9 — [Java module declarations](https://docs.oracle.com/javase/specs/jls/se25/html/jls-7.html#jls-7.7).**
  A dedicated declaration centralizes module dependencies and package exports.
  Java also checks public access on declarations; WF instead puts its full
  public declarations in the one interface without a second publication switch.
  [Package naming and access](https://docs.oracle.com/javase/specs/jls/se25/html/jls-7.html#jls-7.1)
  separately show that hierarchical names need not grant parent/child access,
  and that filesystem organization does not eliminate declaration-name conflicts.
- **E10 — [Haskell export lists](https://www.haskell.org/onlinereport/haskell2010/haskellch5.html).**
  An export list selects declared/imported entities without duplicating their
  definitions. This is a comparator for the superseded thin-list candidate,
  not the selected self-contained handwritten interface.
- **E11 — [OCaml interface checking](https://ocaml.org/docs/compiler-frontend).**
  Explicit interfaces are checked against implementations. This supports the
  selected full-declaration mechanism as a legitimate checked design, not as
  an inherently unverified boundary. WF's contract and nominal correspondence
  rules still need specification and evidence.

## Discriminating validation criteria

These criteria precede any experiment for selecting this design. No performance
measurements have been made for this proposal.

### Interface, module and collaboration qualification

Verify that a caller can determine every public signature, capability, effect
and proof clause from the handwritten interface and explicit dependency
interfaces, without implementation browsing or generated missing clauses.
Use ordinary complete declarations, including generic/function-kind APIs.
Preserve GrowVector's actual proof requirements when evaluating an abstract
interface; its unresolved logical-vocabulary gap is not permission to weaken
the library contract. Record declaration-edit costs, reading errors, interface
and private dependency-selection conflicts and cross-module coordination before
claiming collaboration benefits. No such trial has been performed.

Require negative witnesses for absent/duplicate implementations, mismatched
labels, modes, bounds, effects or contracts, public references to private
definitions, conflicting dependency roots and cycles involving private
dependencies. Changing only an implementation requirement must diagnose a
correspondence failure. Interface-only declaration checking must not authorize
lowering without a checked implementation. A private helper in one file must
be usable from another without a public declaration or per-file import.

Permute directory enumeration and top-level item order; preserve local lexical scope
and reject constant/group cycles and invalid recursive layouts by their actual
rules. Move a function between files, add a private file, and change one public
declaration with its implementation. Compare checking, correspondence and
backend query counts plus current-source diagnostics. A whole-interface or
build-file fingerprint invalidating all bodies fails precision. Verify public
concrete type identity is shared with implementation uses, then separately
qualify abstract representation/capability correspondence and public logical
expressions before claiming a representation-hiding API.

Compare an indivisible LLVM module per WF module with compiler-owned backend
partitions on the same source and optimization policy. Count frontend/proof
reuse separately from LLVM work: retaining source judgments while regenerating
a whole module object is not fine-grained backend reuse. Record object counts,
link cost, peak memory and missed inlining/layout opportunities. File moves
must not define new semantic compilation boundaries; debug-info updates may
still require output changes. Runtime quality and required edit precision,
not the number of object files alone, select the backend grouping.

### Namespace and dependency qualification

Exercise the paired `vector.wfm` / `vector/*.wf` and
`vector/other.wfm` / `vector/other/*.wf` layout. Parent compilation must not
collect child implementation files; an unmatched implementation directory must
not acquire an ancestor owner. Check that canonical directory enumeration,
member addition/removal, duplicate declarations, namespace/declaration clashes
and ambiguous root/path bindings have deterministic results across supported
hosts. No implementation filename introduces another namespace.

Require a direct grandchild import and a cross-subtree `a/b/c -> d/e/f` import
when d precedes a at their divergence, including prefix directories with no
`.wfm`. No intermediate API forwarding is needed. Reject the opposite branch
direction and a child-to-parent dependency even in isolation, with a diagnostic
naming the violated ancestor/order boundary rather than an import-graph cycle.
Check self-import, missing/duplicate/unknown order entries, root-order bypasses
and canonical alias ambiguity. Public and private edges use the same rule.
Validate transitive-use and familial-private-access failures independently.
An attempted cycle must contain an individually illegal edge under every valid
tree order; changing names alone cannot make all its edges legal.

Retain the acyclic two-branch interleaving counterexample as a qualification
witness for the policy's extra restriction. Before selecting the dependency
certificate, compare ordered subtrees, distributed module ranks and a central
module sequence on matched dependency additions, removals and reversals with
unchanged APIs, including that interleaving witness. Count renamed module paths,
changed imports and qualified uses separately from edited order/rank entries,
shared configuration conflicts and revalidated permission queries. An existing
permitted edge needs no order edit; a real cycle must remain impossible. Do not
claim a locality improvement if it only moves widespread source edits into
widespread rank edits. Representative consumer and concurrent-writer costs are
unmeasured; the structural proof alone does not select a certificate layout.
The strict postorder argument establishes the abstract policy's acyclicity;
the bounded model check does not certify a future resolver or root loader.

Check invalidation separately for interface/body edits, directory inventory
changes, child-order changes and module-path renames. An unrelated ancestor API edit must not
recheck a client's source proofs merely because the imported leaf has that
ancestor's path prefix. Real namespace lookup and optimizer dependencies still
count. Changing a relevant ancestor's child order must revalidate affected
imports; inserting an unrelated sibling must not invalidate proofs merely by
shifting conceptual ranks. Per-directory queries must not put a whole source-
root or architecture-file digest into every module key. Compare explicit
generic policies and client adapters with shared lower contracts; verify that
moving a provider while retaining a forbidden higher dependency still fails.

### Correctness and dependency precision

For a fixed specification, compiler, target, source snapshot, dependency
selection, declared architecture order and optimization policy, compare a fresh build with builds reached
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
field selection and hidden paths in public declarations reject, interface
capabilities must match checked representations, and construction/destructuring
cannot bypass linear release or existing opacity. Cache eviction and an unrelated failed module
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
decision with structurally acyclic module dependencies under a declared finite
order, including private dependencies; its coupling to namespace subtrees is
under reconsideration for change locality. It replaces name-resolution's
global inventory and top-level order dependence with
filesystem-qualified modules, direct directory ownership, shared local names
and complete checked public interfaces. The declared certificate constrains
permitted edges without creating dependencies or access privileges.
The proposal also clarifies the compiler root's artifact boundary and adds one
compiler child for persistent incremental computation. The constitution's safety and
performance priorities, fixed deterministic proof families, generic
specialization and finite-cycle rule, callable contracts, source ownership,
backend fact obligations, self-tail semantics and runtime model remain in
force. The current generic scratch-analysis rejection is retained: different
verification contexts cannot share proofs merely because IDs are stable.

Four uncertainties are implementation acceptance work, not weaker endpoints:

- Verify interface/source grammars, exact declaration correspondence, public
  semantic closure, abstract representation/capability matching, order-independent
  formation and exact specification deltas. Qualify canonical root/path rules,
  direct directory membership, namespace/declaration collisions, complete child
  orders and the three-case import predicate. Qualify deep imports, ancestor
  rejection, root binding and ordinary parent/child privacy. Compare explicit
  module ranks and a central module sequence for dependency-change locality;
  the ordered-tree rule is stricter than arbitrary DAG acceptance. A separately compiled module
  private to a parent subtree remains a possible capability with no selected
  rule; reopen only for a concrete consumer that cannot use one module's private
  files. Qualify explicit private dependency selection and interface comparison;
  measure collaboration before claiming an advantage.
- Establish a compositional soundness argument for claim rebinding and
  component receipts, including cycle edits, deletion and failed-build cases;
  differential edit-sequence tests alone do not prove soundness.
- Qualify complete LLVM planning dependencies and choose fragment/optimization
  region granularity using measured runtime quality and edit costs, including
  the single-LLVM-module comparison without conflating frontend and backend reuse.
- Demonstrate useful cold and warm costs on real module consumers. Start with
  the existing GrowVector library/caller separation, then modularize the
  existing wfgrep and SHA-256 programs without changing algorithms, and add a
  generic-heavy multi-module consumer plus controlled dependency-graph scaling.
  These are proposed consumers, not currently implemented module benchmarks.

The maintained TODO links these unresolved capabilities and criteria. This
design review can judge the architecture and its stated limits; it cannot
certify an unimplemented incremental checker or claim unmeasured performance.
The prior review's private-contract composition finding remains unresolved;
the complete-interface refinement neither supplies a logical-view mechanism nor
treats that finding as closed.
