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

The proposed dependency authority is one project-root graph file containing
ordered module declarations and their exact direct dependencies. Directory
paths still determine names and implementation ownership, but no longer
determine dependency direction. Earlier-only references certify acyclicity;
changing the declared graph does not itself rename modules. The complete
proposed grammar is qualified separately from its semantic implementation.

The [complete source specimen](demo/README.md) makes this direction concrete:
five modules form a fixed-capacity job queue, a function-kind batch consumer,
an allocation-free entry and a heap-using tool. It includes every interface
and body, the root graph, a reading order, expected behavior and predicted
edit effects. The selected declaration, observation and footprint rules are in
[LANGUAGE.md](LANGUAGE.md), and [SYNTAX.md](SYNTAX.md) contains the complete
strong-LL(2) candidate. The specimen is not an executable project or evidence
that the compiler implements those judgments.

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
Self-containment is relative to explicitly named external interfaces, not
duplication of every dependency's API into one file. External references are
fully qualified or use aliases declared in that same interface file; dependency
permission comes only from the root graph file, not a second import list.
Compiler inputs for layout, specialization and optimization are a separate
concern.

This requirement selects full public declarations over the earlier thin
export-list candidate. Checked repetition of a function declaration is an
acceptable cost of an independently readable contract; minimizing repeated
text does not meet the requirement by itself. Ordinary function-body and
implementation-only helper edits need not change the interface file. A public
struct's private representation does live in that file under the single-definition
choice below; changing it can edit `.wfm` without publishing a field. Module size
still does not select incremental checking or LLVM partition size.

Retain a declaration-only `.wfm` so that public-contract reading and review
remain separate from executable body edits. Putting public function
bodies there would remove header duplication but mix those changes in the same
file. For the current discussion, assume one agent implements each module;
coordination among that module's `.wf` authors is outside scope and supplies no
ground for field-access rules. Agent assignments are workflow assumptions,
not compiler-visible identities or language-enforced editing permissions.

| Candidate | Benefit | Cost and disposition |
|---|---|---|
| Per-source module/import headers and `pub` declarations | Definition and visibility appear together | Refused: the public contract is spread across implementation files |
| Thin export list plus generated complete interface | Each signature and contract is written once | Superseded: the handwritten file alone does not satisfy public-interface self-containment |
| Complete public declarations in one `.wfm`, with checked ordinary implementations | A caller or agent can read and hold the written contract fixed independently of implementation | Selected; the compiler must enforce declaration correspondence and the implementation must prove the declared obligations |
| Complete public function definitions in `.wfm`, private definitions in `.wf` | One written signature and contract per function, with centralized publication | Not selected: implementation edits would enter the public-contract file; fine-grained incrementality does not itself require either placement |
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

Each source module has exactly one `module.wfm` inside its owning directory.
The directory's root-relative path owns the module name; the interface owns
complete public declarations and any file-local
aliases needed to read their external references. Top-level declarations and
struct fields default to module-private. Only `.wfm` permits the `public`
modifier, on declarations and struct fields at their publication sites;
`.wf` cannot publish anything independently. There is no `private` modifier,
separate export list or implementation-side publication switch. An alias is
still only a file-local abbreviation and cannot carry `public`.
Public function declarations
include generic parameters and bounds, parameter/result labels and modes,
types, capabilities, effects, and complete `requires`/`ensures` clauses.
No executable function body belongs in `.wfm`, including a getter used by a
public contract. Its public declaration is here and its checked body is in
`.wf`, without repeating `public`. Every public source-defined struct has its
one complete definition here, including unmarked private fields; its field
schema is a data declaration, not an executable body.
Public constants and interface/binding groups include their public definitions.
Private supporting types and constants needed by those definitions also live
here, without `public`. Closed enum cases, private payloads and complete group/binding publication
follow the selected rules in LANGUAGE.md.
No interface fragment, textual include, wildcard export or forwarding
alias can fill in an omitted part from an implementation file. Alias headers
are local name bindings rather than public declaration items; placing one in
`.wfm` does not publish a second name for its target.

The sole project-root graph file fixes the primary source root by its own
directory and names each module's direct dependencies. External package
binding is outside this selected implementation. The filesystem rule below determines
module paths and direct implementation membership within that source snapshot;
there is no separately editable module-name or member-file map. Neither
`.wfm` nor `.wf` repeats a dependency-permission list. Adding an implementation
dependency therefore need not rewrite the public interface.

One direct-dependency set applies to both the public interface and private
implementation. An edge permits reference to the target's public declarations;
it does not re-export them or grant private access. Whether an edge is used by
a public signature, an implementation, both, or neither is determined from
checked references, not a second writer-maintained `api`/`impl` classification.
A dependency used only in implementation remains absent from the handwritten
public API. The graph cannot inject declarations, rename qualified source
references or change their meaning through per-module aliases. Each canonical
root has one selected identity. Version acquisition and package solving remain
outside the language.

Illustrative contents of `counters/module.wfm`, not an accepted grammar. The path
already declares the module, so no second written module name is needed:

```text
public fn advance(value: own u64) -> next: own u64 pure contract {
  requires value < 18446744073709551615_u64;
  ensures next == value + 1_u64;
};
```

An ordinary implementation in one selected `.wf` repeats that function's
declaration without the publication modifier and supplies its body. Other
selected files may define private helpers without mentioning them in `.wfm`.
There are no per-file module,
import, export or namespace declarations.

Interface formation can check names, types and contract well-formedness
without implementation bodies. That does not prove an implementation exists
or satisfies the interface. Lowering and accepted composition still require
current checked implementations and the existing recursive summary rules.
An interface file is neither a theorem assumption nor an object-code promise.

### Checked interface and implementation correspondence

Resolve `.wfm` declarations first, including private type support, then collect
the complete local implementation inventory. A function definition matching
a callable declared in `.wfm`, whether public or private, binds to the
interface's stable identity; it does not introduce a second function or
overload. Exactly one selected definition must implement each required declared
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

Type, constant and group definitions in `.wfm`, including private support, are
directly available to the owning implementation inventory. A struct is defined
once, never redeclared or extended in `.wf`; its complete field list and
capability modifier determine the ordinary component capability obligations.
Generic capability derivation and imported summaries still need qualification,
but there is no split source representation to match. Function correspondence
remains necessary. The existing `opaque` modifier is not repurposed, since it
already restricts construction even in its declaring scope.

### One module namespace and declaration formation

All implementation files form one module namespace and one local declaration
inventory. A helper defined in one file can be used by another, including
mutual function recursion, without an interface entry, import or export.
File boundaries are locations for authorship and diagnostics, not privacy,
shared-declaration visibility or source linking boundaries. File-local alias
headers provide the separate abbreviation scope described below. The public
interface selects which of these identities external modules may use; it does
not mediate internal references.

Each module's own implementation has a flat local inventory. Directory paths
organize modules into qualified namespaces as specified below; a child
directory's `module.wfm` introduces another module, not another file in the
parent's private scope.
No separate `namespace` block is introduced. Internal code sees its own
module-private declarations, and local spelling collisions still need ordinary
diagnostics. Splitting a file inside the same module does not grant privacy.

Local uses resolve against the full inventory; dependency uses have an
explicit root-qualified path such as `pkg::counters::advance`, possibly
abbreviated by a file-local alias. Module roots, aliases and local names have
unambiguous ownership. There is no wildcard import, implicit
transitive import, overload search or cross-module namespace extension.
Selected source/dependency identity, canonical module path and local declaration
identity determine a nominal, not its printed path without the selected root.
Implementation filenames do not determine declaration identities. Re-exports
and export renaming remain unselected; a stable facade over independently changing
modules is the concrete consumer that would reopen that choice.

Collect all module declaration names before resolving definitions. Functions,
nominals, constants and interface/binding groups are visible independently of
file/item order. This changes TYPE-6's lexical visibility of non-function top-level
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

The root graph has one architecture and may declare multiple build targets.
Each executable target pairs an entry with its environment requirements,
including an optional no-heap requirement. Neither a reusable module interface
nor the whole architecture graph owns that requirement. The target-selection
and checked heap-closure rules below separate source correctness from whether
one particular executable requires the heap.

### File-local name aliases

Introduce one header form, illustrated here before ordinary declarations:

```text
alias vec = pkg::containers::vector;
alias Vector = pkg::containers::vector::Vector;
alias append = pkg::containers::vector::append;
```

The file can then use `vec::append`, `Vector` or `append` in the roles their
targets already admit. These are name bindings, not dependency declarations,
new nominal types, function wrappers, generic specializations or textual
macros. Alias resolution retains the original declaration identities and source
locations; it does not rewrite the parsed tree into another source program.
All generic arguments, named call arguments and ordinary proof obligations
remain those of the target.

Aliases occur only in a file's initial header, before all ordinary items, and
are visible throughout the remainder of that file. They do not enter the
module's shared declaration inventory. Different implementation files may use
the same alias spelling for different targets; their actual functions/types
still belong to the shared module inventory. There is no block-local alias,
module-wide alias side file or implicit inheritance from `.wfm` into `.wf`.
An interface must declare its own abbreviations, rather than obtain them from
implementation files. This keeps reading and concurrent editing local.

Every alias target is a complete root-qualified path beginning with the fixed
`pkg` qualifier or an explicitly selected external dependency name.
Resolution binds that root in the source's owning-package context before forming
the canonical identity. It names a module or a source declaration spelling,
with no file-local alias on the
right-hand side, relative-parent search, wildcard, grouped import, supplied
generic argument or arbitrary type expression. There are no alias chains or
cycles to resolve. A module/function/constant alias uses IDENT; a nominal,
constructor or interface/binding-group alias uses TYPEID. Primitive types,
operation-table rows, built-in numeric bounds, locals, fields, labels and proof
invariant names are not alias targets.

A declaration alias preserves the target path's entries in the ordinary
grammar-selected name domains. A struct spelling therefore retains both its
nominal and constructor entries; an enum type alias does not automatically
bind its variants. If one legal canonical spelling has distinct entries in
different domains, the alias preserves that distinction rather than resolving
by expected type. Owner-dependent member labels are unchanged: aliases do not
rename fields, named arguments, payload-field variant labels or result labels.
Wrong-class uses receive the ordinary domain error. Duplicate alias names
reject, and every occupied domain obeys the existing collision/no-shadowing
rules. The `pkg` qualifier cannot be rebound, and module aliases cannot
shadow an external dependency root available in that source context. They
occupy the file's IDENT binding space but have no value or callable use.

Targets are checked even when an alias is unused. A foreign target requires
the owning module's direct edge and public visibility; a local target uses
the current module's ordinary visibility. In `.wfm`, local targets must be
declared in that same interface. Each use of a module alias still checks the
canonical final owning module: naming a parent grants no child-module edge.
Removing a dependency while an alias still names it is a source error, just
as retaining an ordinary qualified use would be. Aliases never repair a
missing graph edge, expose a private declaration or re-export the target.

Interface and implementation declarations may choose different aliases or
full paths. Correspondence compares resolved identities and normalized
contracts, not the abbreviation spelling. Alias renaming alone creates no
new nominal identity, generic instance, ABI name or runtime work. A changed
target can change meaning and must invalidate its actual lookup consumers.
Moving a definition between files preserves identity but must preserve or
rewrite its aliases to keep the same resolved body.

Rust `use ... as ...` supplies a useful name-binding comparator [E12]. WF's
candidate deliberately has file scope, one header form, no glob/group imports
and no public re-export. The spelling `alias` avoids overloading WF's existing
`use` proof-step keyword; the two contexts could be parsed separately, so this
is a clarity choice, not a claim of unavoidable ambiguity. The complete
qualified-name grammar still requires qualification before implementation.

### Fixed current-package qualifier

There is one active `modules.wfg` for a build. Its directory is the primary
source root, without a declaration such as `root app = ".";` and without a
choice between application, library or project-name prefixes. Use the fixed
qualifier `pkg::` for the current source package. A package here is a selected
source root and its dependency identity; it is not a module, executable
target or LLVM compilation unit. Both a library and an executable's sources
use the same spelling. The abbreviation names the package role without
adopting Rust's term for its compilation unit. Merely having a source root
does not create a module node or grant any dependency edge.

In the active graph, `pkg::` denotes its primary root. In a `.wfm` or `.wf`,
it denotes the selected root that owns that source. Thus a library selected
under a dependency name such as `math` still writes `pkg::vector` internally;
that use resolves to the selected library's vector module, not the importing
application's vector module. This context comes from explicit source ownership,
not a search for a nearer graph file. The active graph still registers all
selected external modules and their exact edges; importing a library does
not import another graph as a dependency authority.

An external source dependency retains an explicit name such as `std` or
`math`. Its binding selects an actual dependency identity, not a second
identity for each spelling. External names and the source's owning-package
context must be recorded resolver inputs. The complete external-binding
format, including dependency-name environments, multiple selected versions
and references to the primary root from another root, remains qualification
work; the single-package demo does not claim to exercise that system. All such
selection must remain explicit in the one graph, and no name binding grants
a module edge. The compiler-owned prelude remains distinct from an ordinary
source library called `std`. External-library selection and library-to-library
dependencies are deferred beyond the next implementation, which covers the
primary package's module DAG. No external binding syntax, version resolver or
graph-import tooling is selected here; reopen that design when external-library
composition is explicitly selected.

The directory of the unique graph already answers where the primary root
starts. A freely chosen local prefix therefore added a naming decision without
solving that problem. A fixed current-package qualifier [E13] instead distinguishes
own-source references from named dependencies and lets library-internal names
survive selection under a caller's dependency name. It is not needed for the
DAG proof. Own-package root-qualified paths use this one form rather than a
second implicit-root spelling; ordinary local names and file-local aliases
remain available.

### Filesystem namespace paths and module ownership

Within a selected root, `vector/module.wfm` declares `pkg::vector`, and
`vector/other/module.wfm` declares `pkg::vector::other` in that root's source
context. The directory path alone supplies the name; `module.wfm` is a fixed
interface filename, not a namespace component. There is no independent
module-name declaration or directory-to-namespace remapping. Distinct selected
dependency instances retain distinct nominal identities even when their
relative directory paths are equal.

```text
vector/
  module.wfm
  core.wf
  growth.wf
  other/
    module.wfm
    core.wf
```

The direct `.wf` records beside `vector/module.wfm` implement `pkg::vector`.
The direct `.wf` records beside `vector/other/module.wfm` implement
`pkg::vector::other`. Neither the
child interface nor its implementation is part of the parent implementation.
Do not recursively collect `**/*.wf`. A selected implementation record must
have `module.wfm` in its immediate owning directory; an unmatched
record cannot silently inherit some distant ancestor's ownership. A directory
prefix such as `a/b/` may organize `a/b/c/module.wfm` without either
`a/module.wfm` or `a/b/module.wfm`:
such prefixes name locations, not implicit modules or dependency graph nodes.
A source root may itself contain `module.wfm`; only an explicit graph row for
that root module registers it. The qualifier alone does not do so.

Only the in-directory fixed filename is an interface form. Do not search for
the old sibling `vector.wfm` or the repeated-name `vector/vector.wfm` as an
alternative. Keeping the interface beside its bodies makes a module's own
files one review/work location; a fixed basename avoids another name to update
when its directory is renamed. It costs filename-only navigation: diagnostics,
links and review surfaces need the relative path to distinguish interfaces.
Moving a directory also moves its child modules; an agent's ownership of the
parent module is its interface and direct implementation files, not recursive
ownership of the entire subtree. Rust's two filename forms [E14] are a useful
comparison, not a compatibility requirement for WF.

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
in its parent module: `vector/other/module.wfm` can coexist on disk with an
exported function called `other` in `vector/module.wfm`, but that conflicting namespace is not
accepted when both modules are registered in the architecture graph. The
namespace inventory records its module rows and namespace prefixes independently
of which build target is selected; modules absent from that graph do not enter
the inventory merely because their files exist. The inventory does not infer access privileges from a component's
existence. Canonical path/case/alias rules must yield the same names on
supported hosts, with ambiguity diagnosed rather than resolved by filesystem
iteration order. Their
complete acceptance spelling remains part of grammar/input qualification.

Moving a function between direct files of the same module preserves semantic
identity. Moving or renaming a module path changes its qualified identity and
requires affected imports and uses to change. Moving only the bound physical
root while retaining its selected source identity and relative paths does not
rename every module. Path-derived naming deliberately trades free module
relocation for one inspectable relationship between source layout and names.

### One root file for the explicit module graph

The required property remains stronger than discovering a cycle among source
imports: the declared architecture must certify source-module acyclicity
through simple consistency and per-edge checks. The dependency graph is now
independent of the namespace tree. Keep canonical path-derived names, but
declare all source-module edges in one file at the project root.

Use `modules.wfg` as the project-root filename. SYNTAX.md qualifies the
complete proposed grammar. The graph has an implicit primary root and one
ordered declaration for each module in the selected graph. Each declaration lists exact direct dependencies, all of which
must have been declared earlier. The following is illustrative notation,
not accepted build grammar:

```text
pkg::shared::memory: [];
pkg::vector::other: [pkg::shared::memory];
pkg::vector: [pkg::shared::memory, pkg::vector::other];
pkg::kernel: [pkg::shared::memory];
pkg::tools::image: [pkg::shared::memory, pkg::vector];

target kernel {
  entry pkg::kernel::start;
  no_heap;
}

target image_tool {
  entry pkg::tools::image::run;
}
```

The selected schema is one or more module rows followed by zero or more named
targets, with the complete formation rules in LANGUAGE.md. No source-root
binding is written. A target names one entry and optionally `no_heap;`; it
cannot change module edges. Target names are build labels, not source aliases.
The graph directory and normalized paths determine one unambiguous module
inventory independently of host enumeration or invocation directory.

All rows belong to one architecture graph, including disconnected modules.
Its canonical identities, namespace inventory, exact permissions and order
certificate do not vary by target. Checking a library graph needs no target
or entry. Building an executable selects exactly one declared target; its
entry is a public ordinary function in a registered module. Ordinary generic
and runtime argument binding and result interpretation still belong to the
build invocation under FN-7/PROG-3, with the ordinary startup obligations and
no new entry-signature restriction. The entry is not a source-file alias.
PRE-1 remains the compiler-owned implicit inventory under its existing rules;
it is not a hidden writer-selectable source module.

Each row supplies two distinct pieces of information: its listed edges are
the declared graph, and its position certifies those edges' direction.
Merely appearing earlier does not grant another module access. A bare ordered
list without adjacency would not expose which dependencies the program
actually declares.

For this candidate, the graph file is the sole source of module-dependency
permission. Source files contain qualified references or file-local aliases,
not additional imports that create or repeat graph edges. There are no
wildcard edges, automatic transitive imports, per-directory dependency files,
includes, conditional edge expressions or dependency-generating scripts in this graph
format. The build selects one target from the graph, or checks selected modules
without constructing an executable; neither action rewrites the architecture.
The build cannot add hidden source edges through another channel. These
restrictions concern the source graph, not the compiler's separate
semantic/optimization queries.

### Build targets and the no-heap boundary

One graph serves the kernel and the heap-using image tool in the example.
Separate graph files per executable would duplicate architecture and could
make individually acyclic permissions cyclic in their union. Separate
independent projects may have separate graphs; several outputs of this project
do not require that split. A target declaration in the same file makes the
entry/requirement pairing inspectable without creating another dependency
authority. Its complete proposed syntax is qualified in SYNTAX.md; target semantics are in LANGUAGE.md.

Distinguish three sets. The architecture graph contains every registered
module and is always validated structurally. A target's source composition
contains its entry module and the transitive closure of declared direct
dependencies, including dependencies used only by implementation. Every
definition in those modules receives the ordinary required source judgments,
including unused nongeneric functions and symbolic generic schemas. A
check-only invocation selects module roots and their dependency closure;
checking all registered modules includes disconnected components as well.
Finally, the target's execution closure follows concrete callable instances
from its bound entry and startup, with function-kind actuals, ordinary call
edges, compiler-derived releases and required native/runtime implementations.
Source correctness is not reduced to that last set.

The no-heap requirement is checked against that conservative execution closure
and its required value/layout descriptions before optimization. It withdraws
the heap capability: heap-bearing values and allocating operations cannot be
introduced by an entry argument, private representation, callee, concrete
generic binding, release path or required native/runtime implementation.
The physical descriptions behind an abstract public type participate in this
check. Inlining, LLVM dead-code elimination or a guessed runtime branch cannot
select whether the requirement passes. The finite closure and summary rules
must be specified, deterministic and complete for their admitted family;
selected compiler-owned native/runtime supplies must account for their heap
requirements under the existing SCOPE-3 trust boundary. A writer's `pure`
annotation or an absent body does not establish those requirements.

This intentionally revises STOR-8's compilation-unit-wide spelling ban into
a target requirement over checked composition; it is not merely relocating
the old declaration. A heap-using helper outside the selected execution
closure does not by its presence reject a heap-free entry, even if the two
share a source module. The helper still receives every ordinary safety and
formation check. A syntactic call in the selected conservative closure cannot
be excused because a later optimizer might remove it. Implicit availability
of an allocating prelude declaration is not a call either.
Object and runtime selection must also avoid requiring heap infrastructure
solely for unselected helpers. Merely checking source reachability and then
linking an indivisible object that requires an allocator is not sufficient
for the kernel consumer; qualify emission/section selection and runtime supply
alongside the semantic check.

Heap capability is a compiler-computed, dependency-tracked summary, not a
target-dependent meaning for an ordinary function. A reusable module has one
interface and one set of checked source facts. A heap-enabled tool can reuse
heap-free code; selecting its target does not grant heap access to the kernel
target. Changing only a target requirement rechecks composition against current
summaries instead of redoing every source proof. Reuse still requires the same
relevant specification, compiler, machine and layout inputs; removing an entry
label from a key does not erase an actual target-dependent domain fact.
LANGUAGE.md fixes startup, layout, native-supply and release coverage;
implementing and qualifying it remains work. No implemented no-heap verifier
for this target model is claimed.

### Local validation and the DAG argument

Validate the graph against the selected canonical source snapshot:

1. The implicit primary root and each registered module path are unambiguous.
   Every selected module has one canonical identity and exactly one graph
   declaration, resolving to its directory's `module.wfm`;
   canonical aliases cannot create distinct positions for one module.
2. Each dependency list names exact modules without duplicates. Every target
   must already have a valid declaration in this same graph. Self, forward
   and unknown references reject at the row and offending edge.
3. Every external source reference, after file-local alias resolution, must
   name a public declaration in an explicitly listed direct dependency.
   Both interface and implementation references obey this rule. A missing
   edge rejects even if the target row
   occurs earlier or is transitively reachable.
4. The selected program is closed over declared dependencies and all selected
   definitions receive ordinary checking, including unused definitions.
   Every selected module participates in this same certificate; path or
   native-binding spelling supplies no bypass.

Only insert a module into the earlier-declaration lookup after validating its
row. If `position(M)` denotes its declaration position, every edge `A -> B`
then has `position(B) < position(A)`. A directed cycle would strictly decrease
positions and return to its starting position, which is impossible. Any closed
subset of this declared graph inherits acyclicity.

Every finite DAG has a dependency-first declaration order: a nonempty DAG has
a module with no outgoing dependency edge among the remaining modules; declare
one such module and repeat. This is a representability argument, not an order
inference step performed by the compiler. Fixed directory grouping therefore
adds no further restriction to which acyclic module graphs can be written.

The compiler need not infer a topological order, search for a module cycle,
or discover the graph by inspecting implementation bodies. It still parses
the declarations and checks every explicit edge. With a canonical-name lookup,
this is one pass over module/edge records plus name-resolution cost; no
measured speedup is claimed. Actual consumed semantic dependencies and
concrete generic/callback FN-6/FN-9 graphs still need their own analysis.
The source-module DAG is not the function, proof or optimizer graph.

Declaration position is a certificate, not module identity, a promise that
the module's body is correct, or a mandate for serial compilation. Independent
modules may compile in parallel. An earlier declaration grants neither
unlisted access nor unchecked implementation evidence.

### Paths, parents, children and sharing

A module may directly depend on any explicitly listed earlier module,
regardless of directory distance. Thus `a/b/c -> d/e/f` requires an earlier
`d/e/f` row and an edge in `a/b/c`'s row; no directory move or intermediate
forwarding is needed. A parent may depend on a child, or a child on a parent,
when that target is earlier and explicitly listed. The file cannot permit both
directions simultaneously. Ancestry alone grants no dependency or private
access; prefixes without `.wfm` are still namespaces rather than modules.

The former ordered-subtree counterexample becomes directly expressible:

```text
pkg::a::a2: [];
pkg::b::b1: [];
pkg::b::b2: [pkg::a::a2];
pkg::a::a1: [pkg::b::b1];
```

Both `a/a1 -> b/b1` and `b/b2 -> a/a2` are legal with unchanged paths. No
complete subtree has to come before another. Shared providers can have many
incoming edges without relocation into a special `shared` namespace. Moving
a provider into such a directory is an organization choice, not an acyclicity
repair.

A new edge to a later module requires a compatible reordering within the
graph file. If no order works, the requested graph is cyclic and the desired
dependencies or module boundaries must change. Lower shared contracts,
explicit generic policies, client adapters or a combined module remain
possible API-specific responses; changing the file alone cannot resolve
a genuine cycle or find the correct business abstraction. A cyclic graph
cannot become legal through ordering, aliases or native linkage.

Parents and children retain ordinary interface privacy. Java's package
hierarchy [E9] is evidence only for separating names from private access, not
for this proposed DAG certificate. A separately compiled module private to a
parent subtree remains an unselected visibility capability with its own
reopening consumer.

### Dependency edits, source stability and collaboration

All direct-dependency declarations and their order live in this one root
file. Changing the declared graph does not require moving source directories,
renaming modules, editing distributed ranks or synchronizing a second import
list. The statement concerns dependency metadata: adding a new call, changing
a public type or replacing an API can still require the corresponding source,
interface and proof edits. Removing an edge with remaining source uses must
reject those uses rather than silently admitting an undeclared dependency.

One file is not a constant-size diff. Existing edges may require multiple rows
to move, and concurrent agents can edit the same graph. Keep one explicit row
per module, no hand-written numeric ranks, and preserve unrelated ordering
rather than globally sorting the file after each edit. Textual merges still
require ordinary graph validation. These choices reduce avoidable churn; their
effect on collaboration has not been measured.

A source-path rename still changes its canonical identity and affected source
references. The graph cannot act as a namespace alias or prevent that ordinary
cost. In contrast, an order-only edit preserves identities, public declarations
and source references. Incremental queries must separate graph formation,
per-module adjacency, edge-order validity and semantic consumers: a whole-file
hash or changed numeric position must not invalidate all body proofs.

The graph file makes dependency declarations inspectable in one place. A
reader of a `.wfm` still sees its complete public signatures, contracts and
external references, including every file-local alias, without implementation
browsing. The owning-package context and explicitly selected dependency
interfaces resolve those references; the graph supplies the root selections
and permissions, not missing declaration text or implementation-only names.

### Alternatives and selection grounds

| Candidate | Useful property | Cost and disposition |
|---|---|---|
| Filesystem descendants only, or whole ordered subtrees | Tree relationships certify acyclicity | Superseded: acyclic cross-branch edits can force path/namespace changes across consumers |
| Numeric ranks in individual module files | Local strict-decrease check, independent of directory grouping | Not selected: order repair can spread edits across module files and separates adjacency from its order certificate |
| One central order plus dependency lists in source files | Names stay stable and order is explicit | Not selected: a graph change still has multiple writer-maintained locations |
| One root ordered adjacency file | Explicit edges and a locally checked order in one place; paths remain independent | Selected proposal: no distributed ranks or repeated import lists; shared-file editing and row movement remain costs to qualify |
| One root unordered adjacency file plus cycle detection | Explicit graph without ordering edits | Viable under a weaker requirement, but not selected while the architecture must provide a checked order rather than infer one |
| Bare central order granting every earlier module | Very short configuration | Not selected: expresses possible directions but hides the intended direct dependency graph |

The [earlier ordered-tree argument and exploratory enumeration](https://github.com/mbbill/Whitefoot/blob/17a5fbbf01397975801d957aa590b9c1173e556e/research/investigations/modular-compilation/DESIGN.md#why-local-checks-guarantee-a-dag)
remain evidence only for that superseded predicate, not this file format,
a future resolver, collaboration costs or performance. The new candidate
rests on the strict declaration-position argument above and needs its own
grammar/resolver and edit-locality qualification.

### Public semantic closure, representation and proof paths

A public declaration must be understandable using its interface, explicit
dependency interfaces and the ordinary language/prelude rules. Publicly usable
signatures, fields, bounds, effects and proof expressions cannot require a
caller to name a private type, helper, constant, group or representation path.
Private field declarations in `.wfm` may use private support declared in that
same interface. Private implementation contracts may use private paths normally;
their presence as text in `.wfm` does not make those paths legal public contract
vocabulary.

Every nominal that a caller must name belongs in `.wfm`, even when its fields
are private. Type publication and field publication are separate choices:

| Record form | Definition location | Implementation and caller access |
|---|---|---|
| Module-private implementation struct | One complete definition in `.wf`, without a visibility modifier | Available to the module's shared private inventory; cannot supply missing type information to `.wfm` |
| Private supporting struct needed by an interface field | One complete unmarked definition in `.wfm` | Available throughout the owning module, inaccessible to external source |
| Public struct with private fields | One complete `public struct` definition in `.wfm`, with only externally accessible fields marked `public` | The owning module sees all fields; external source sees only explicitly public fields |
| Fully public data record | One complete `public struct` definition in `.wfm`, with every field marked `public` | Ordinary field operations and construction remain subject to existing rules |

The type itself being public is enough to require its full source definition
in `.wfm`, even if every field is private. A public name-only declaration with
a separate `.wf` representation is no longer selected. Every field and its
order occur once; no public-field repetition, private field extension or
complete-versus-projected-schema marker is needed. Existing fieldless native
and prelude opaque types retain their own rules, not a new source construction
route.

For example, a public buffer may declare a public caller-controlled `tag` and
unmarked private storage in the same definition. A private storage helper used
by that field also needs its complete definition in `.wfm`, without `public`,
with any private supporting constants. Resolve this closure from the same
interface, the ordinary prelude and permitted public dependency interfaces,
never from `.wf` or another module's private names. A public field or callable
signature cannot expose that private helper type. Marking a field `public`
inside a private type does not publish the enclosing type or create an external
access route; effective access also requires an accessible enclosing type.

Use private fields for state maintained through operations and public fields
for independently usable data. A struct with all fields private provides the
abstract API without a second representation definition. Existing `readonly`
and `opaque` retain their meanings; neither is an access-visibility modifier.
Public enum, payload, constant and group rules are selected in LANGUAGE.md; their compiler qualification remains implementation work.

One authoritative type definition with explicit publication replaces both the
whole-record-only choice and the split public-projection candidate. A public
projection plus full `.wf` definition duplicated fields and required a
correspondence rule; private-only extensions required a merge/layout rule.
Both are superseded. Default-private also makes adding representation detail
and expanding the public API distinct edits; default-public with `private`
would publish an unmarked addition. The cost is more markers on public data,
and private representation edits now change `.wfm`; private support can make
that file larger. Privacy restricts source access, not whether a reader sees
representation text. Reading and incremental costs remain unmeasured; one
agent per module remains the authoring assumption.

Universal getter/setter mediation is not selected. A setter for each private
field does not by itself preserve a multi-field invariant; operations such as
append or reserve can preserve it as one checked change. For public records,
ordinary projections also express component ownership and disjoint borrows
directly. Under REF-3 a getter cannot return a reference, so an all-accessors
policy would need different consuming/callback APIs, not a mechanical rewrite
of field access. Hiding representation still needs useful precise public
effects; treating every operation as touching the whole object can lose
independence. No new type-invariant assumption follows merely from privacy.

Field publication is a compatibility and incremental-dependency choice:
changing a public schema can affect caller source proofs and ABI consumers;
changing a private field in `.wfm` with unchanged consumed capability, ownership
and contract facts can preserve source proofs while invalidating layout,
release and codegen consumers. A private-field edit may also change capabilities
or residual-release eligibility and then must recheck those source consumers.
Do not key every client by the whole `.wfm` digest. Private access does not
guarantee an unchanged ABI or that every accessor call will inline. These
effects need the consumer evidence below.

The contract-composition gap is addressed by the selected rules in
[LANGUAGE.md](LANGUAGE.md#integer-observations-and-executable-getters): a
marked getter has a checked finite affine realization in `.wf`, an opaque
public integer observation in contracts, precise named read support, and
explicit current/entry/result views. Named footprints expand private effect
paths without publishing field access. The GrowVector wrapper/function-kind
witness and queue result construction exercise those rules. This closes the
previous unspecified-design P2; implementing and testing the judgments is
still required, and this prose is not a soundness proof of a compiler.


Self-containment concerns the source API, not all compiler information.
Compiler-owned artifacts retain checked implementation evidence, generic
bodies and private layout/release descriptions for specialization, by-value
representation and optimization. A client author need not read those bodies.
Source privacy creates no mandatory boxing, dynamic dispatch, runtime checks
or destructor escape. A layout change may invalidate code generation even
when the publicly usable API is unchanged; dependency interface changes can
also alter resolved API meaning without editing this module's `.wfm`.
Composition validates those actual identities and inputs.

### Selected representation boundary

[LANGUAGE.md](LANGUAGE.md) owns the complete proposed rules for field access,
construction, residual release, derived capabilities, closed public enums,
constants/groups, observation admission and state, named footprints, and
absence of implicit type invariants. The examples below explain the already
selected field/capability choices. Their acceptance cases belong to the
implementation matrix, not a remaining menu of undecided language alternatives.


### Field visibility and structural operations

The publication and single-definition choices above supersede the earlier
split-definition discussion. The selected operation policy separates access
permission from ordinary ownership obligations; it introduces no new `readonly`
meaning or object-invariant assumption. This is proposed module behavior, still
requiring specification and executable qualification.

Direct public fields have the existing field semantics for copy reads,
references, writes and in-place operations, with ordinary
ownership, effects and `readonly` restrictions. Expose independent user data,
such as a buffer's caller-controlled tag, this way. A changing length related
to hidden storage can remain private and be observed through an ordinary
public getter; admitting that getter in contracts is still separate work.
Existing TYPE-2 `readonly` forbids ordinary source writes inside the module
as well as outside it, so it cannot quietly become an external-only access
restriction. A public writable field does not establish a private invariant.

For a struct with every field public, ordinary construction retains its existing
rules. If any field is private, external construction goes through declared
ordinary functions, not aggregate construction with guessed
or defaulted private fields. Inside the defining module the complete definition
permits normal construction. This does not repurpose TYPE-2 `opaque`, whose
constructor restriction also applies in the defining scope.

Under OWN-1/WIN-3, a field move consumes its whole owner, releases the other affine parts and
rejects if a remaining linear part cannot be consumed. The whole type's
copy/drop pair alone does not describe the remainder: a non-droppable type
could owe that property to the extracted public field, a hidden field, or an
explicit modifier. Previously, a private definition outside `.wfm` could change
that remainder without a readable interface change. That ground for banning
every external extraction no longer holds: all fields and their type support
are now declared in `.wfm`, even though external code cannot name private paths.

Permit direct public-field moves and consuming destructuring when ordinary
residual-release checks pass. An external consume can bind only accessible
fields; every unbound part, private or public, must satisfy the existing release
rules. A final `..` covers omitted private fields without granting access or
waiving their consumption obligations. A private linear remainder rejects;
a public consuming function can handle it inside the module. Changing a private
field's capability changes a tracked source-operation dependency, not an
invisible implementation fact. This retains useful consuming access without
a separate readable/writable/movable visibility family or a blanket ban on
mixed-field types. Whole-value moves, permitted release, ordinary updates and
swaps keep their existing rules. A copy field is read bare, not moved, and all
existing borrow, effect, opacity and capability-modifier checks still apply.

A minimal witness owns a public affine queue and a private `u64` tag. Moving
the queue consumes the containing owner and can discard the tag; replacing
that tag with a private linear resource makes the same extraction reject.
A reference into the consumed owner becomes invalid; using it later rejects
under REF-2. Destructuring an opaque operand still rejects under TYPE-2.
Validate these distinctions, ordinary construction
only when every field is accessible, and corresponding incremental invalidation
before claiming compiler support.

### Public capabilities

Modules do not add a second handwritten copy/drop contract. Reuse TYPE-2/OWN-1/PROV-6 derivation from owned components and declaration
modifiers, and export checked derived facts rather than repeat a capability
pair in source. This extends existing rules to the proposed boundary; no new
capability family or module-dependent ownership class is proposed.

Private fields participate exactly like public fields. A struct containing
only integers remains copy and drop even with every field private. `nocopy`
removes copy when that is the intended restriction; `nodrop` removes both
copy and drop. These modifiers restrict the same type inside and outside its
module. Neither privacy nor a public factory creates an ownership capability,
and copying a value is distinct from invoking its field constructor.

For an otherwise unmodified `Holder<T>` owning just one field of type `T`,
the concrete instance has the capabilities of `T`, irrespective of that field's
visibility. A parameter contributes through components actually owned, not
merely because it appears in the nominal's argument list. A written `T: copy`
or `T: drop` is an admission requirement and supplies facts during symbolic body
checking; an absent bound supplies neither. A concrete copy instance does not
retroactively license copying in an unbounded generic body. A bound on `T`
also cannot make a container copy if another owned component prevents it.

`nodrop` requires explicit consumption; it does not require a particular named
finalization function. Legal structural consumption remains a route under
PROV-6, including when the enclosing nominal is nodrop but the omitted fields
are droppable. A genuinely linear private remainder instead prevents external
structural disposal. Any stronger protocol restricting consumption to one
operation would be a separate design question, not an inferred benefit of
privacy or nodrop. Existing compiler-derived release runs no user destructor.

The expected benefit is one authority for capabilities and no manual formula
to keep synchronized. The cost is that a private field can change a public
type's derived capabilities or residual-release conditions, requiring public
surface reporting and precise invalidation. Compare concrete and generic
instances across a dependency before choosing any additional capability
annotation; a derived API display can improve readability without becoming a
second source definition. Logical observations and the decision to retain explicit operation contracts are specified in LANGUAGE.md.

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

Explicit publication also gives CI a useful review signal. Compare resolved
public surfaces between the reviewed base and head: added/removed declarations,
effective visibility, field types, generic bounds, callable labels/modes/types,
effects, normalized contracts, and public constant meanings. Include changes to
resolved dependency identities and derived capability or permitted-operation
facts even when no line containing `public` changes. A keyword-only diff can
draw attention to edits but cannot establish API stability: changing a public
field's type, a contract on following lines, or a referenced constant can leave
the modifier untouched.

Report representation/layout/release changes separately when they preserve
the usable source API, since they still affect backend consumers. A private
field edit can also change source capabilities or extraction rights and then
belongs in the semantic report. Qualification needs witnesses for both cases,
plus alias renaming with unchanged identity and alias retargeting with changed
identity. The expected benefit is focused review without flagging every private
body edit; the cost and completeness of a resolved comparison are unmeasured.
Defer CI wiring until the module frontend supplies the checked metadata. This
is a review aid, not a new language acceptance or repository approval rule, and
this design does not add a CI script or a checked-in API snapshot.

### Module collaboration and incremental ownership

An implementation task can hold the public `.wfm` fixed while changing several
files in the module's shared namespace. Public signature changes edit both
interface and corresponding function definition; implementation-only helper/file
changes need not edit it, while a public struct's private fields live there.
Direct directory membership is a tracked source input; dependency declaration
changes touch the root graph file, where
concurrent edits can conflict. Neither duplication checks
nor a flat namespace proves improved
multi-agent throughput. Cohesive contracts and independent changes remain
better module boundaries than a fixed file/line count or an exclusive agent lock.

Cache interface declarations, implementation declarations, correspondence,
body checks and lookup results separately. Adding a private file checks its
definitions and affected lookup/scope consumers, not all module bodies.
Reordering the canonical source inventory changes no semantic identity. Moving
a definition between files in the same module preserves its declaration
identity. Proof reuse additionally requires the same resolved body, including
its new file's alias bindings; source maps and debug-information consumers may
still change. Track used aliases, missing-name results and collision checks,
not one whole-file alias-table hash in every body key. An added unused alias
can still change a collision/no-shadowing result and must not bypass that check.
Never key every body or object by the entire `.wfm` or build-file digest.

Validate each external reference against the source module's normalized
direct-dependency row and the graph's earlier-target checks. Inserting or
moving an unrelated row may change declaration positions without changing
edge validity or identity. Revalidate affected graph checks; unchanged
semantic claims retain their ordinary reuse conditions. Do not put a row
position or whole graph-file digest into every body-check key. Listed edges
are the declared dependencies, while consumed semantic and optimizer inputs
remain separately tracked. File order does not force serial compilation of
modules that do not actually depend on one another.

## Compilation units and file boundaries

One source module is one logical compilation unit: it has one public interface,
one private declaration inventory, and a checked composition of all selected
definitions. Several implementation files are inputs to that unit. No file
needs its own exported header, object identity, source import or native link
step just to call a helper in another file.

This does not choose the units of compiler computation:

| Boundary | Selected responsibility |
|---|---|
| Source file | Storage, editing, parsing, local alias bindings and source coordinates |
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
| Module surface / lookup | Owning-package identity, explicit external bindings, path components, public/private inventories, relevant file aliases, direct dependency paths, lookup role and spelling | Stable resolved declaration or diagnostic |
| Graph formation / edge validation | Root graph bytes, canonical module inventory, exact adjacency rows and earlier-target relations | Resolved roots, stable per-module dependency sets, target declarations and valid order certificate, or located graph diagnostic |
| Target composition / heap requirement | Selected target, bound entry, declared module closure, concrete call/layout/release/native summaries and target requirement | Checked source composition, current execution closure and satisfied environment requirement, or diagnostic |
| Module dependency permission | Canonical source/target identities and membership in the source's normalized adjacency row | Allowed direct dependency or missing-edge diagnostic |
| Interface correspondence | Resolved public declaration and selected implementation declaration | Matching identity and checked normalized declaration, or diagnostic |
| Contract / type shape | Resolved declaration, arguments, capabilities and projections | Normalized semantic boundary |
| Template check | Symbolic body, bounds, callee boundaries and summary availability | Symbolic checked body |
| Concrete body check | Body, complete substitution and semantic query results | Typed body, ownership/effects and obligations |
| Proof component | Current component membership, obligations and predecessor summaries | Verified clauses and derivations |
| Allocation / call summary | Current local seeds and call edges | Specified fixed-point summaries |
| Layout / lower | Checked body, machine target, relevant layouts and release descriptions | Target-qualified IR |
| Optimization plan | Summaries, visibility, prevailing definitions, profiles and policy | Imports and per-partition decisions |
| LLVM backend | Own/imported IR, decisions, toolchain and machine-target settings | Optimized object and remarks |
| Final link action | Selected objects, runtime, linker/options and entry | Executable; ordinary full link when inputs change |

Module is the source/distribution boundary; a function or concrete instance is
the ordinary body-check boundary; a recursive component is an atomic proof
publication boundary. Query granularity is not forced to whole modules. An
edited file can be reparsed while unchanged item values stop downstream
invalidation. Token-level editor parsing is not necessary to avoid checking
untouched files and bodies.

Graph parsing projects roots, per-module edges and individual target records
separately. The source's selected owning root and the external bindings it
actually resolves are tracked inputs: identical text such as
`pkg::vector::Vector` in two packages must not select one cache identity.
Conversely an importer's name for a library does not by itself rename all
internal declarations when the selected identity and bindings stay the same.
Source checking produces reusable declarations, proofs and heap
requirements; target checking reads those results and the selected entry's
ordinary argument binding. A target name or no-heap flag is not a blanket
body-proof key. Changing an entry changes its selected composition and closure;
changing a requirement revalidates the affected target against current
summaries. An edit to a private representation or a previously heap-free
callee can invalidate that result even when the target's text is unchanged.

Re-evaluate affected queries and compare their result values before invalidating
consumers. A changed body with the same verified callable boundary does not
change ordinary call-site checking inputs. Track negative lookup results,
export-set queries and candidate eligibility too: a previously absent name or
unprofitable inline candidate can become relevant. Do not place the hash of
every imported module's entire source in every consumer key.

Separate stable identity from revision. Declaration identity uses selected
module identity, declaration domain and name, never an alias spelling;
item-local node identities plus the current source map recover diagnostic
locations. Concrete instances add the complete normalized type, const and
function argument vector. Dense FunctionId/NominalId
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
generic templates throughout the selected module composition. Checking source
correctness only for entry-reachable bodies would weaken present acceptance;
the separate target heap requirement does not grant that exemption. Required
concrete instances are checked separately. No final
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

The root graph's explicit edges and declaration order certify source-module
acyclicity through local checks. Both interface and implementation references
require a listed edge; neither can introduce another source-graph direction. Shared
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
directory decomposition is expressible by independent module rows in the root
graph while preserving that requirement.

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
composition receipt and satisfied target environment requirements. Its inputs
include the selected entry, build target, machine target, libraries,
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
| Only adjacent/descendant imports or ordered namespace subtrees | Tree structure certifies acyclicity | Superseded: dependency changes can require unnecessary regrouping and path changes |
| Arbitrary cross-subtree imports plus cycle detection | Permits any acyclic grouping | Superseded: the required architecture must certify every permitted edge set before graph analysis |
| Distributed ranks or a central order separate from source imports | Local strict-decrease proof without subtree direction | Not selected: dependency metadata still has several authorities; use one root ordered adjacency file |
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
| PROG-1/2/3, FN-7 | One ordered bundle, no modules, build-selected unqualified entry | One graph's directory fixes the primary source root; explicit external-root selection, ordered adjacency and named entry targets share that graph; source composition follows declared module closure while ordinary startup obligations remain required |
| FORM-2/3, GRAM-1/2/3/4/5, DIAG-1 | One root and unqualified name roles | Complete interface/source and root-graph forms, file alias headers, qualified names and diagnostics joining graph rows, aliases, declarations and definitions |
| TYPE-6, CONST-2, FN-3 | Whole-unit identity; non-function top-level visibility follows source order | Directory-named modules with fixed module.wfm interfaces and shared local names; pkg resolves to the source's owning root, external names select explicit dependencies, and only permitted direct interfaces are visible; ordinary privacy, dependency validity and lexical local scope retained |
| Public declaration correspondence / type representation | No separate interface or public/private source boundary | Declarations and struct fields default private; only .wfm permits public. Every public source struct has one complete definition there, with private support; functions retain declaration-only interfaces and exact normalized body correspondence without repeating public |
| Type/ownership/release consumers | Descriptions in one inventory | Same judgments over imported descriptions; privacy grants no storage or release exemption |
| FN-2/4/6/9, ENT-3.S12 | Whole-unit instances and summary identities | Same instance and SCC rules across modules, with current cached claims and availability |
| DIAG-2 | One exact-program value owns/discards all evidence | Checked component fragments and assembled receipt; failed composition grants no authority, unrelated valid entries survive |
| STOR-8 | A no-heap unit rejects forbidden type/call spellings throughout its source | An entry target withdraws heap capability from its conservative concrete call/value/layout/release/native closure; ordinary checking still covers every definition in selected modules |
| STOR-6, EFF-3, PAR-1/2 | Whole-program target/allocation/parallel metadata | Same rules over complete tracked layout, allocation and call-summary dependencies |
| FN-8/9, ENT-2/3, EFF-1/3 | Ordinary function calls are excluded from contracts; pure is not a termination guarantee | Finite affine observe functions retain runtime bodies in .wf and supply checked current/entry/result views; named footprints preserve private effect precision; ordinary pure remains no termination promise |
| PRE-1 / native binding | Compiler-owned declarations and linked bodies | Bind selected prelude, runtime and target identity into composition/codegen inputs |

The candidate uses each directory's fixed module.wfm as its module declaration
and puts complete public declarations with explicit external references and
any file-local aliases in that file.
There is no second module-name declaration, source import list,
implementation-side `pub` or namespace block. Direct directory membership
determines implementation records. The one project-root graph file fixes the
primary root by its location and lists ordered modules
with their exact direct dependencies and named entry targets. Current-package
references use the fixed `pkg::` qualifier in the owning source context.
The graph is named `modules.wfg`. LANGUAGE.md specifies its formation,
namespace/privacy, correspondence, observations, footprints and target rules;
SYNTAX.md supplies the qualified complete grammar. External bindings remain
outside the single-package scope. META-5 deltas and the active version are
computed against the actual integration base; no count is invented here.

| Current implementation owner | Required structural change |
|---|---|
| `source.rs`, syntax/canonical rendering | Canonical selected roots, direct directory snapshots, interface/source and graph grammar roots, stable path/item identity and source maps |
| `resolution/engine*` | Owning-package and external-root resolution, canonical path namespaces, root-graph adjacency and earlier-target validation, file-local aliases, direct-dependency permission checks, shared local inventories, public closure, correspondence and positive/negative lookup dependencies |
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
  rules are selected in LANGUAGE.md and still need compiler evidence.
- **E12 — [Rust use declarations](https://doc.rust-lang.org/reference/items/use-declarations.html).**
  A use declaration creates synonymous local bindings and supports explicit
  renaming. Rust also permits module/block scopes, grouped and glob imports,
  and public re-exports. WF borrows the name-binding purpose, not that complete
  scope/visibility system: aliases have file-header scope and grant no graph
  edge, publication or new type identity.
- **E13 — [Rust crate qualifier](https://doc.rust-lang.org/reference/paths.html#crate).**
  `crate` resolves from the current crate's root. This is a comparator for one
  fixed own-source qualifier, not a reason to adopt Rust's other path forms,
  crate build boundaries or package machinery. WF uses the spelling `pkg`
  and retains its one explicit module graph and separately tracked
  checking/backend partitions.
- **E14 — [Rust module filenames](https://doc.rust-lang.org/reference/items/modules.html#module-source-filenames).**
  Rust permits a named sibling source file or a directory's `mod.rs`, but not
  both for one module. WF has no compatibility requirement to retain two forms;
  its proposed `module.wfm` puts the complete public contract beside the direct
  implementation files. Repeated interface basenames remain a navigation cost.

## Implementation contract and sequence

The source-language design is selected in [LANGUAGE.md](LANGUAGE.md); its
complete [syntax candidate](SYNTAX.md) is checked by the native grammar
qualification below. No source-package resolver, general termination prover,
implicit type-invariant mechanism or incremental native linker is needed to
start. They are not missing pieces of this implementation contract.

### Responsibilities and concrete values

Keep the current safe-Rust checker and backend process boundary. Extend the
existing driver and semantic ownership points; do not build a second modular
checker. The following are responsibility boundaries, not mandatory new files.
Read the corresponding live compiler nodes before implementing their structure.

| Owner | Inputs and outputs | First existing consumer to change |
|---|---|---|
| Source/graph session | Immutable input snapshot, `ModuleKey`, direct source inventory, graph rows/targets | `compiler/src/driver.rs` and source formation |
| Syntax and resolution | Role-aware `.wfm`/`.wf` trees, factored qualified syntax, file aliases, canonical `DeclKey` | `compiler/src/syntax` and semantic declaration formation |
| Interface index | Public claims, complete nominal schemas, footprint paths, derived capability projections, correspondence result | Nominal/call formation and visibility checks |
| Observation checking | Admitted view DAG, domain/read support, realization receipt, current/entry/result view substitution | `semantic/check/requires.rs`, `ensures.rs`, `support.rs`, calls and result transfer |
| Dependency query store | Stable keys, canonical values, recorded reads, reverse edges, status and receipts | Existing checker entry points called by the driver |
| Component scheduler | Current ordinary-call SCCs and generic/function-binding closure, predecessor availability | `semantic/check/publication.rs` and generic finiteness |
| Proof fragment store | Local proof nodes plus imported claim slots; checked composition map | Current checked-program publication and audit/remapping |
| Lowering/backend | Per-instance checked IR, layout/release keys, stable symbols and fragment plans | `compiler/src/backend/emitter.rs`, ABI and generated helpers |
| Native construction | Pinned LLVM planner/worker process, optimized objects, final link manifest | Existing launcher/native build path |

Use these logical key shapes (typed serialized tuples, not concatenated names):

```text
ModuleKey = (package input namespace, normalized module components)
DeclKey = (ModuleKey, declaration domain, optional type owner, declared name)
InstanceKey = (DeclKey, ordered resolved type/const/function argument vector)
ClaimKey = (InstanceKey, clause role/ordinal, normalized relation)
ViewKey = (proof context, observer InstanceKey, scalar images, value images, support versions)
QueryKey = (query family/schema, subject key, explicit configuration inputs)
Receipt = (producer QueryKey, canonical result, dependency results, local evidence)
ObjectKey = (fragment identity, IR, backend plan, imported bodies, target/toolchain)
```

In this single-package implementation the package input namespace is the
canonical selected graph location, scoped by compiler/specification identity.
Moving a whole checkout may miss this version-private cache; it cannot alias
another root. Within that namespace, physical source filenames, graph row
positions, aliases and dense arena ordinals do not enter declaration identity.
Function-local node paths identify proof occurrences only within their
canonical body revision. Statement insertion may recheck that function; it
must not renumber unrelated functions. No persistent statement identity
matching heuristic is required.

A stored query record contains schema/version, key, canonical input/result
values, consumed dependency keys and values, typed payload, and a complete
publication marker. Hashes address records; canonical equality resolves
collisions in acceptance-bearing records. Decode/schema/checksum failure is a
miss. Publish by temporary write plus atomic rename after success. One in-flight
producer per key coalesces parallel demand. Cycles go through the specified
component query, never recursive memoization that treats an unfinished result
as success. A bounded memory cache and disk reclamation may discard results;
resource policy changes work performed, never source acceptance.

At each build: capture inputs; validate graph and selected source membership;
form interfaces and inventories; resolve/check correspondence; discover finite
instances and current components; validate/recompute demanded body and proof
queries; compose current receipts; lower/plan dirty fragments; reuse or rebuild
objects; perform an ordinary native link and publish only a complete result.
Interface formation can overlap independent work. A source module DAG does
not impose serial LLVM execution or replace the concrete proof graph.

For a queried record, recursively validate its recorded dependency results.
If they are equal, reuse. Otherwise recompute with the ordinary producer while
recording a fresh read set; an equal resulting semantic value stops downstream
invalidation. Retire removed edges, negative lookups and obsolete witnesses.
Do not replace the old valid executable after a failed prospective build.
Freshly validated independent results may still enter the cache.

Source input detection is an explicit cost: a fresh CLI process reads/hashes
the selected source snapshot and directory inventory before trusting its content.
Timestamps alone and advisory file watchers are not acceptance authority.
A future verified content-snapshot provider can avoid rereading bytes, but no
such service is required or claimed here. Warm checking can therefore have
linear input-validation I/O while doing zero unchanged parsing/proof/backend
work. Report that I/O separately instead of calling the entire no-op build
constant-time.

Observation realization and public meaning have separate dependencies. An
external proof uses the abstract observation identity, admitted domain/support
and checked public guarantees. A module proof that expands a view additionally
reads its realization. Editing the getter's body revalidates its realization
and every expansion consumer. If the public claims and their current
availability remain identical, other callers can rebind current evidence;
code containing or importing the old executable body must still regenerate.
Footprint changes revalidate effect, overlap and view-support consumers even
when source signatures have identical text.

### LLVM construction protocol

Start with one concrete function plus inseparable local helpers per bitcode
fragment, with private constants/release helpers assigned explicit owners.
Use the same optimized path on cold and warm builds. A persistent planner
consumes LLVM summaries and symbol resolution through a pinned LLVM-version
backend process. A worker request names its own bitcode, exact imports,
exports/internalization, prevailing definitions, target/features, optimization
pipeline and profile content. Its successful response is the optimized object
and dependency/diagnostic metadata. Rust consumes files/protocol values rather
than linking unsafe LLVM FFI into the acceptance checker.

Expose planner stages as queries: symbol resolution, current call/reference
components, live/export sets, cost/import candidates including rejected
candidates, prevailing definitions, per-fragment import/internalization plans,
and worker actions. Each enabled LLVM planning transform must declare all its
read inputs; a truly global analysis gets an honest global query with result
comparison. Cross-module import and specialization are enabled from the first
native modular experiment. Never cache solely by the previous import list.

The implementation includes an LLVM-version-specific planner adapter where
upstream APIs do not expose the required read set/state. Stock ThinLTO's full
index rebuild is useful as a differential oracle, not the delivered endpoint
for incremental optimization planning. No stable cross-LLVM protocol or remote
worker service is selected. Changing LLVM invalidates its artifacts.

Full-LTO is a runtime-quality comparator. If a representative workload loses
an optimization because a fragment boundary hides information, improve the
summary/import transform or join the affected optimization region; keep the
source/checking units unchanged. Use measured cost to choose larger groups,
not a permanent module-sized LLVM unit. Global invalidation when a real global
fact changes is valid; omitting that dependency is not an optimization.

### Ordered implementation slices and completion evidence

Each slice advances the same final system; none is a reduced completion target.
Do not mark the full work finished after merely generating several objects.

| Slice | Implement | Required discriminating evidence |
|---|---|---|
| 1. Normative boundary | Reconcile this branch's v0.62 grammar with the exact integration base, amend/archive the active spec once, implement graph/source roles, names, publication, correspondence, complete nominal ownership | Grammar and parser cases; wrong-role, missing-edge, private access, duplicate definition and graph-order negatives; all existing single-bundle cases retain their stated behavior under the selected entry path |
| 2. Abstract APIs | Footprints, finite affine observations, state/result views, explicit view expansion, public enum/group rules | Queue factory/client; GrowVector external wrapper and function-kind actual; stale-state, hidden linear remainder, exact-effect, unproved realization and recursive-observation negatives |
| 3. Query and proof persistence | Canonical keys, recorded read sets, local proof fragments, current SCC availability, atomic receipts and source-map remapping | Cold/warm edit sequences with equal verdicts, changed cycle/edge deletions, failed producer and corrupted cache miss; same source observations with optional optimizer facts disabled |
| 4. Shared specialization and targets | Build-wide instance ownership, layout/release projections, entry binding, target heap closure and runtime selection | One instance requested by several modules; kernel/tool shared module including unused allocating helper; hidden heap and called generic actual negatives; no allocator symbol required by kernel output |
| 5. Optimized fragments | Stable symbols, one prevailing definition, cross-module imports, complete persistent plans and cached optimized objects | Inline-body and formerly rejected import edits; layout/ABI changes; recursive helper and scheduler/runtime ownership; clean/warm executable equivalence |
| 6. Cost and quality | Explicit paired experiments on real programs plus controlled scaling | Separate input/check/proof/plan/backend/link costs, affected queries and object counts, peak memory/cache size; account for each repeatable runtime loss against the same-image and full-LTO comparisons |

Preserve a legacy source-bundle CLI as a driver-formed synthetic single module
feeding the same checker during integration. It cannot combine with a graph,
act as a second visibility policy inside a graph, or bypass the new checks.
Its compiler-owned synthetic interface derives the existing entry signature
from that same input inventory and grants no cross-module source exports. The
legacy `program no_heap` input keeps its current source-wide formation rule
in this entry mode; graph sources reject that spelling. This is an explicit
input compatibility rule feeding the same checker, not an alternate cache or
proof path. It does not permit `.wfm` bodies. Retire this compatibility entry only as a separately
explained interface change, not by weakening existing conformance coverage.

The implementation must also provide an explicit `check` operation over all
registered modules without requiring an executable target, a named-target
build operation, a way to disable cache reuse for differential verification,
and a machine-readable query/phase report. CLI spellings belong to the driver;
source acceptance and graph meaning cannot depend on them. No daemon, package
server, installation manager or distributed build service is required.

### Grammar qualification

Run explicitly from the repository root under the ordinary construction guard:

```sh
perl .github/run-check.pl module-grammar sh -c 'rustc --edition=2024 research/investigations/modular-compilation/qualify.rs -o /tmp/wf-module-qualify && /tmp/wf-module-qualify'
```

`qualify.rs` copies the current grammar generator into a temporary directory,
changes only its declared production inventory/count and new keyword mapping,
and runs its existing FIRST/FOLLOW, strong-LL(2) and overlapping-token-predicate
checks for source and graph starts. A mutation restoring the unfactored
qualified call/value alternatives must fail the same check. It removes the temporary copies; it changes
no compiler/specification files and is not a daily gate input. The source and
graph candidate passed these checks. This proves the grammar decisions tested,
not lexer/parser implementation, role-specific formation, proofs or execution.
Delete the adapter when the active specification supplies these productions.


## Discriminating validation criteria

These criteria precede any experiment for selecting this design. No performance
measurements have been made for this proposal.

### Interface, module and collaboration qualification

Verify that a caller can determine every public signature, capability, effect
and proof clause from the handwritten interface and explicit dependency
interfaces, without implementation browsing or generated missing clauses.
Every alias used by that interface must be declared in its own header.
Use ordinary complete declarations, including generic/function-kind APIs.
Preserve GrowVector's actual proof requirements when evaluating an abstract
interface through the selected observations and footprints, without weakening
the library contract. Record declaration-edit costs, reading errors, interface
and root-graph editing conflicts and cross-module coordination before
claiming collaboration benefits. No such trial has been performed.

Require negative witnesses for absent/duplicate implementations, mismatched
labels, modes, bounds, effects or contracts, public references to private
definitions, conflicting dependency roots and cycles involving private
dependencies. Changing only an implementation requirement must diagnose a
correspondence failure. Interface-only declaration checking must not authorize
lowering without a checked implementation. A private helper in one file must
be usable from another without a public declaration or per-file import.

Require equivalent interface/implementation signatures with different local
aliases, and rejection when an alias instead names a different nominal or
callable. Exercise module, function, constant, type/constructor and group
aliases in their actual grammar roles, including generic calls, matches and
contracts. Check file isolation, same short name with different targets in two
implementation files, missing graph edges, alias chains, duplicate names,
wrong-case/domain aliases, private targets, shadowing and attempted re-export.
An alias in `.wfm` must not become a public member or silently enter a `.wf`.
Owner-dependent field/argument/payload labels must keep their declared spelling.

Reject executable bodies in `.wfm`, including getter bodies, and reject `public`
anywhere in `.wf`. Exercise a public record whose fields are individually public,
a mixed-field struct and a public struct whose complete field list is private.
Keep private support in `.wfm`; reject missing support in `.wf`, inaccessible
types in public signatures/fields, a `public` alias and duplicate/extended struct
definitions. A public field of a private enclosing type must not publish it.
Qualify direct reads, writes, disjoint field borrows and the selected consuming
operations, including droppable and linear private remainders. Reject private
field access and external construction needing private fields. Derive and check
all capabilities rather than trusting a type's publication modifier.
Retain ordinary `opaque`/`readonly` behavior and REF-3 rather than synthesizing
reference-returning getters. Compare private layout changes against public
field-schema changes for semantic, layout, release and codegen invalidation.
Qualify by-value allocation-free use instead of forcing handles or claiming
unmeasured accessor inlining. The getter/contract witness below remains required.

Permute directory enumeration and top-level item order; preserve local lexical scope
and reject constant/group cycles and invalid recursive layouts by their actual
rules. Move a function between files, add a private file, and change one public
declaration with its implementation. Compare checking, correspondence and
backend query counts plus current-source diagnostics. A whole-interface or
build-file fingerprint invalidating all bodies fails precision. Verify public
concrete type identity is shared with implementation uses, then separately
qualify imported capability/release facts and public logical
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

Exercise `vector/module.wfm` with `vector/*.wf` and
`vector/other/module.wfm` with `vector/other/*.wf`. Reject the former sibling
and repeated-name paths as alternative interface lookup forms. Exercise an
explicitly registered root module and an unregistered namespace-only root.
Parent compilation must not
collect child implementation files; an unmatched implementation directory must
not acquire an ancestor owner. Check that canonical directory enumeration,
member addition/removal, duplicate declarations, namespace/declaration clashes
and ambiguous root/path bindings have deterministic results across supported
hosts. All graph-registered modules and their prefixes enter namespace lookup,
independently of the selected target; adding an unregistered module on disk must not silently add a declaration or
name collision. No implementation filename introduces another namespace.

Check that neither an application nor a library declares a local root nickname:
both use `pkg::`. Reject attempts to rebind that qualifier. Qualify graph
paths against the primary root and source paths against their selected owning
roots. Reuse unchanged library sources under different consumer dependency
names without rebinding internal paths to those consumers. Complete and test
external dependency-name environments, multiple selected instances and access
to the primary root from another root before claiming external-source
composition. Imported graph files must not add edges or select a second root
implicitly; std naming does not import the entire prelude or grant edges.

Require a declared direct grandchild dependency and a cross-subtree
`a/b/c -> d/e/f` edge when the target row is earlier. Exercise child-to-parent
and parent-to-child dependencies in separate valid graphs; ancestry grants
neither a permission nor a prohibition. Admit the two-branch interleaving
example without moving directories. Reject missing/duplicate module records,
missing/duplicate/forward/self edges, unknown paths, ambiguous roots/aliases,
hidden source imports and unlisted use of an earlier or transitive module.
Public-interface and implementation references use the same adjacency
authority. Preserve private-access failures independently of graph validity.

For any attempted cycle, at least one edge must fail the earlier-target check
under every declaration order. Validate that the checker never infers or
silently repairs ordering. Test a valid explicit edge to an earlier module and
an invalid reference to an unlisted earlier module separately: order is not
adjacency. A graph edge also does not make the target's implementation proved.

Compare matched graph additions, removals and reorderings with unchanged API
definitions, including the former subtree interleaving witness. Confirm that
all dependency declarations are edited in the root file, with no repeated
source import list or distributed rank. Count graph rows moved, concurrent
textual conflicts, source edits required by actual API use, and revalidated
queries separately. A single-file declaration does not imply a one-line diff,
no merge conflicts or a solution for truly cyclic requirements. Consumer-scale
coordination costs remain unmeasured.

Exercise check-only module selection and all-module checking, graphs with no
targets, and multiple targets sharing one graph. Build a no-heap kernel and a
heap-using tool over the same heap-free library without duplicate APIs or
dependency maps. Target selection must not change module identity, namespace
collisions or permissions. Each selected source module's unused definitions
still receive normal safety checks; disconnected unselected target modules
are not silently folded into that executable's source composition.

Check that a heap-using helper outside the selected conservative execution
closure does not reject a no-heap entry, but a reachable private call, generic
actual, heap-bearing representation, entry argument or derived release does.
Include required native/runtime supplies and confirm that a source-only
summary cannot hide their heap requirement. A potentially executable call
cannot pass just because LLVM later deletes it. Unused implicit prelude
availability alone is not a call. Entry/target/heap-policy edits must reuse
unaffected ordinary proofs while validating the newly selected composition;
changing a callee's heap requirement must invalidate the relevant target check.
Working-directory changes must not change root binding, and canonical path
aliases must not let one module appear twice in the graph.

Check incremental invalidation for graph edges, graph row ordering, root
bindings, directory inventory, interface/body edits and path renames. A valid
order-only edit must preserve module/declaration identities and unchanged
source proofs; it can revalidate order checks and source locations. An unused
edge addition must not by itself reprove every body; selected-graph closure
and current checked-composition obligations still apply. Deleting a used edge
must invalidate the corresponding permission/use query even if target APIs
are unchanged. Root/graph-file digests and numeric row positions must not
become all-body cache keys. Changing an ancestor's API has no effect on a leaf
consumer solely because the names share a directory prefix. The strict
position argument establishes the abstract guarantee, not implementation or
performance.

For aliases, compare renaming with the same target against retargeting to a
different declaration. A semantics-preserving rename may update locations but
must not duplicate generic instances or reprove unrelated bodies. Moving a
body to a file with a differently bound alias must change its resolved input;
adding an unused alias that collides with a local must still invalidate the
scope check. These witnesses distinguish identity from spelling and prevent a
blanket claim that alias-only edits never affect acceptance.

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

## Design suitability and implementation evidence

The final proposed boundary is one explicit source DAG, self-contained
interfaces with one representation definition, checked scalar observations
and structural footprints, finer-grained source/proof/codegen queries, and
ordinary native linking. The finite affine observation form deliberately
retains the existing arithmetic proof model; arbitrary recursive mathematical
functions and implicit object invariants would add separate termination and
callback protocols without being required by the selected module consumers.
Named single-path footprints preserve independently addressable private state
without adding runtime handles or a set-valued effect language.

The main engineering risks are complete dependency tracking, safe claim
rebinding after component changes, result-view ownership transport, and the
LLVM planner adapter. Their consumers and qualification controls are specified
above. Runtime and build-time superiority are not established by this design;
the paired experiments are implementation acceptance work. A stable artifact
ABI, remote cache service, package resolution and incremental native linker
remain outside the selected scope.

The grammar candidate has been qualified using the existing compiler generator.
The source demo, abstract-container argument and cold/warm transition matrix
are design evidence; no execution or performance measurements are claimed.
The live specification and live trees are unchanged. Pending amendments name
the root/name/effect/proof and compiler decisions that implementation must
replace or extend. Exact spec rule/token deltas and new version identity are
computed against the integration revision, not copied from this branch after
other language work has merged. The maintained TODO records the implementation
and measurement obligations rather than leaving these choices undecided.
