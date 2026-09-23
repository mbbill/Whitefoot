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
changing the declared graph does not itself rename modules. The format below
is a design sketch, not implemented grammar.

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
module name; its contents own complete public declarations and any file-local
aliases needed to read their external references. Every ordinary top-level
declaration in that file belongs to the public API; there is no separate
export list or implementation-side `pub` switch. Public function declarations
include generic parameters and bounds, parameter/result labels and modes,
types, capabilities, effects, and complete `requires`/`ensures` clauses.
Public constants and interface/binding groups include their public definitions.
Public concrete records and enums declare their externally visible schema
there. No interface fragment, textual include, wildcard export or forwarding
alias can fill in an omitted part from an implementation file. Alias headers
are local name bindings rather than public declaration items; placing one in
`.wfm` does not publish a second name for its target.

The project-root graph file explicitly binds canonical source roots and names
each selected module's direct dependencies. The filesystem rule below determines
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
shared-declaration visibility or source linking boundaries. File-local alias
headers provide the separate abbreviation scope described below. The public
interface selects which of these identities external modules may use; it does
not mediate internal references.

Each module's own implementation has a flat local inventory. Directory paths
organize modules into qualified namespaces as specified below; a child `.wfm`
introduces another module, not another file in the parent's private scope.
No separate `namespace` block is introduced. Internal code sees its own
module-private declarations, and local spelling collisions still need ordinary
diagnostics. Splitting a file inside the same module does not grant privacy.

Local uses resolve against the full inventory; dependency uses have an
explicit root-qualified path such as `app::counters::advance`, possibly
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

The root graph owns the optional module-qualified entry and graph-wide
`program no_heap;` declaration. A module interface no longer owns this
program-wide choice. The constraint covers the complete selected graph,
including dependencies used only by implementation and concrete instances,
under STOR-8's ordinary type and call restrictions. Source uses of prelude
declarations obey those restrictions; the implicit presence of an allocating
prelude declaration is not itself a call. Moving an allocation into a private
dependency cannot hide it.

### File-local name aliases

Introduce one header form, illustrated here before ordinary declarations:

```text
alias vec = app::containers::vector;
alias Vector = app::containers::vector::Vector;
alias append = app::containers::vector::append;
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

Every alias target is a complete canonical path beginning with a bound source
root. It names a module or a source declaration spelling, with no alias on the
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
rules. Module aliases also cannot shadow a canonical source-root name; they
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
accepted when both modules are selected. The namespace inventory records path
components and negative lookups for selected module rows and their namespace
prefixes; unselected modules do not enter that inventory merely because their
files exist. The inventory does not infer access privileges from a component's
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

Use `modules.wfg` as the proposed project-root filename. The complete grammar
and canonical rendering still need qualification. The graph file contains
canonical source-root bindings and one ordered declaration for each module in
the selected graph. Each declaration lists exact direct dependencies, all of which
must have been declared earlier. The following is illustrative notation,
not accepted build grammar:

```text
root app = ".";

app::shared::memory: [];
app::vector::other: [app::shared::memory];
app::vector: [app::shared::memory, app::vector::other];
app::main: [app::vector];

entry app::main::run;
```

The schema is an optional graph-wide `program no_heap;`, one or more root
bindings, one or more module rows, and an optional entry declaration, in that
order. Root paths are explicit strings relative to the graph's project root,
not the process working directory or an environment search path. A root name
and every module-path component use WF's IDENT spelling; implementation
filenames do not become identifiers. Selected roots and paths must have one
unambiguous canonical interpretation on supported hosts; multiple names for
one canonical module cannot evade row uniqueness or change nominal identity.

All rows belong to the selected graph, including disconnected modules; entry
reachability does not remove their required source checking. Checking a library
graph needs no entry. Executable construction requires exactly one public
ordinary function entry in a listed module, with the existing PROG-3 argument
and contract obligations. The graph names the function; ordinary generic and
runtime argument binding and result interpretation still belong to the build
invocation under FN-7/PROG-3, without a new entry-signature restriction. The
entry is not an alias from some source file.
The no-heap declaration constrains the whole selected graph, not just entry-
reachable functions. PRE-1 remains the compiler-owned implicit inventory under
its existing rules; it is not a hidden writer-selectable source module.

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
format. The build selects the graph; that graph declares any executable entry.
The build cannot add hidden source edges through another channel. These
restrictions concern the source graph, not the compiler's separate
semantic/optimization queries.

### Local validation and the DAG argument

Validate the graph against the selected canonical source snapshot:

1. Root bindings are unambiguous. Every selected module has one canonical
   identity and exactly one graph declaration, resolving to its `.wfm`;
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
   Every selected external source root/module participates in this same
   certificate; package, path or native-binding spelling supplies no bypass.

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
app::a::a2: [];
app::b::b1: [];
app::b::b2: [app::a::a2];
app::a::a1: [app::b::b1];
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
external references, including every alias needed to interpret them, without
implementation browsing; reading the graph establishes whether those
references are authorized, not what an otherwise ambiguous public name means.

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
source input; dependency declaration changes touch the root graph file, where
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
| Module surface / lookup | Canonical path components, public/private inventories, relevant file aliases, direct dependency paths, lookup role and spelling | Stable declaration or diagnostic |
| Graph formation / edge validation | Root graph bytes, canonical module inventory, exact adjacency rows and earlier-target relations | Resolved roots, stable per-module dependency sets, entry, program policy and valid order certificate, or located graph diagnostic |
| Module dependency permission | Canonical source/target identities and membership in the source's normalized adjacency row | Allowed direct dependency or missing-edge diagnostic |
| Interface correspondence | Resolved public declaration and selected implementation declaration | Matching identity and checked normalized declaration, or diagnostic |
| Contract / type shape | Resolved declaration, arguments, capabilities, projections and applicable program policy | Normalized semantic boundary |
| Template check | Symbolic body, bounds, callee boundaries, summary availability and applicable program policy | Symbolic checked body |
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

Graph parsing projects roots, per-module edges, entry and program policy
separately. Source type/call checks read the no-heap policy where required;
startup checking reads the selected entry, its signature and the invocation's
bound arguments. Changing only the entry does not change every module's
source judgments, while changing the no-heap policy must revalidate all of
its affected uses, including disconnected selected definitions.

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
| PROG-1/2/3, FN-7 | One ordered bundle, no modules, build-selected unqualified entry | One project-root ordered adjacency file owns roots, optional entry and no-heap declaration; earlier-target checks certify acyclicity independently of path-derived names and direct .wf ownership; checked composition and ordinary startup obligations remain required |
| FORM-2/3, GRAM-1/2/3/4/5, DIAG-1 | One root and unqualified name roles | Complete interface/source and root-graph forms, file alias headers, qualified names and diagnostics joining graph rows, aliases, declarations and definitions |
| TYPE-6, CONST-2, FN-3 | Whole-unit identity; non-function top-level visibility follows source order | Path-qualified modules with shared local names; only structurally permitted direct interfaces are visible; ordinary privacy, order-independent top-level names, dependency validity and lexical local scope retained |
| Public declaration correspondence / type representation | No separate interface or public/private source boundary | Self-contained public semantic declarations, exact normalized callable correspondence, one nominal identity, and checked abstract representation/capability correspondence |
| Type/ownership/release consumers | Descriptions in one inventory | Same judgments over imported descriptions; privacy grants no storage or release exemption |
| FN-2/4/6/9, ENT-3.S12 | Whole-unit instances and summary identities | Same instance and SCC rules across modules, with current cached claims and availability |
| DIAG-2 | One exact-program value owns/discards all evidence | Checked component fragments and assembled receipt; failed composition grants no authority, unrelated valid entries survive |
| STOR-6/8, EFF-3, PAR-1/2 | Whole-program target/allocation/parallel metadata | Same rules over complete tracked layout, allocation and call-summary dependencies |
| PRE-1 / native binding | Compiler-owned declarations and linked bodies | Bind selected prelude, runtime and target identity into composition/codegen inputs |

The candidate uses the .wfm path as the module declaration and puts complete
public declarations with explicit external references and any file-local
aliases in that file.
There is no second module-name declaration, source import list,
implementation-side `pub` or namespace block. Direct directory membership
determines implementation records. One project-root graph file binds source
roots and lists ordered modules with their exact direct dependencies. Its
proposed name is `modules.wfg`; exact graph/alias syntax, declaration terminators,
canonical path/collision rules, abstract nominal/capability syntax and
normalized correspondence remain to be specified. META-5 deltas require the
complete grammars and judgments with strong-LL(2) checks; no count is invented
here.

| Current implementation owner | Required structural change |
|---|---|
| `source.rs`, syntax/canonical rendering | Canonical selected roots, direct directory snapshots, interface/source and graph grammar roots, stable path/item identity and source maps |
| `resolution/engine*` | Canonical path namespaces, root-graph adjacency and earlier-target validation, file-local aliases, direct-dependency permission checks, shared local inventories, public closure, correspondence and positive/negative lookup dependencies |
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
- **E12 — [Rust use declarations](https://doc.rust-lang.org/reference/items/use-declarations.html).**
  A use declaration creates synonymous local bindings and supports explicit
  renaming. Rust also permits module/block scopes, grouped and glob imports,
  and public re-exports. WF borrows the name-binding purpose, not that complete
  scope/visibility system: aliases have file-header scope and grant no graph
  edge, publication or new type identity.

## Implementation entry and remaining semantic work

The selected name/graph design can now be translated into the active
specification and the compiler. That translation is implementation work;
this investigation does not claim that a syntax sketch is already a complete
language amendment. No second checker or temporary trusted-interface path is
needed for the first usable multi-module build.

The remaining responsibilities are concrete:

| Area | Required behavior and evidence |
|---|---|
| Graph and source formation | `modules.wfg` owns roots, ordered exact edges, the optional entry and the whole-graph no-heap declaration. `.wfm` and `.wf` have alias headers followed by their own item forms. Verify all three complete grammars and canonical renderings with the ordinary generator/parser checks. |
| Identity and lookup | Root identity and canonical module/declaration identity are separate from source revision, physical file placement, row position and aliases. Equal spellings in different modules are distinct; moving a body between files requires equivalent resolved aliases for reuse. |
| Public correspondence | Form interfaces without reading implementation bodies; compare implementation headers by resolved type/const/function identities and normalized contracts. A matching declaration still needs its checked body and current composition evidence. |
| Proof composition | Keep FN-6/FN-9 template, instance and recursive-component rules separate from the source DAG. Callers may reuse an unchanged claim only with current availability/evidence; deletion must retract dependencies and rebuild affected fixed points. |
| Representation and abstract contracts | Concrete public types have complete public definitions in `.wfm`; private implementation types stay local. General representation-hiding public nominals additionally need checked capabilities, abstract logical vocabulary and effect/ownership correspondence. The existing gap below is not closed by aliases. |
| Optimization and caching | Use the same query/checker path for cold and warm builds, retain checked generic bodies and physical layouts where needed, and track optimizer dependencies separately. Source module boundaries must not require runtime indirection or prevent cross-module specialization. |

There is a specific syntax obstacle to resolve, rather than an invitation to
use a symbol-aware parser. Current `expr := atom infix_tail? | call` can choose
an unqualified value versus call using short token prefixes. Naively replacing
names by segmented paths makes both a qualified constant and a qualified call
start with `IDENT ::`, so the first two tokens no longer select those arms.
Factor the shared name prefix and its call/value continuation in the complete
grammar, preserving the ordinary distinction from `::<...>` generic arguments,
group-member calls and owner-dependent field/variant labels. A lexical
qualified-name representation is another viable technical alternative, but is
not selected here. In either case run the existing strong-LL(2) and token-
predicate checks before making parser behavior normative; no table counts or
passing prototype are claimed by this design.

The private-contract gap is more than a syntax question. The existing
GrowVector preconditions use `storage.inner.len`, while FN-8 excludes ordinary
accessor calls from contracts. Hiding that field prevents a wrapper or
function-kind formal from stating the same requirement. A recommended research
route is a public logical observation declared in `.wfm`, with a private,
checked, pure total interpretation. Clients could name a length observation
in requirements and invariants without naming a representation field. This
would be proof syntax, not a name alias or an executable getter.

Before selecting that mechanism, establish its typed interpretation, finite
formation/expansion rules, value-state identity, entry snapshots, support and
invalidation on writes. Also define abstract effect/region correspondence:
replacing every precise footprint with `writes(values)` can lose independence
that the current source can prove. Public guarantees need checked realization,
not assumed axioms, hidden ordinary calls or new runtime checks. A concrete
witness must carry GrowVector through an external wrapper and a function-kind
formal, retain its preconditions and useful effect precision, and demonstrate
that a state-changing operation kills the right facts. A by-value abstract
type must still have compiler-available checked layout and capability/release
facts. No observer/region syntax or new automatic proof family is selected
here, and the existing P2 remains open until those judgments are specified.

The first implementation should connect graph formation, qualified/alias
resolution, complete interface correspondence and checked composition to the
existing executable path using a real library and caller. Use the general
query boundaries from the start, then persist them and qualify affected-region
proof and optimized-code reuse on edits. A concrete public-type consumer can
exercise this path while the abstract-contract work proceeds; it is not
evidence that representation-hiding APIs work. The final endpoint retains the
full proof, abstraction and optimization requirements rather than treating
that first consumer as completion. Runtime quality, cache precision and
multi-agent coordination require observations from implementation, not more
unmeasured design prose.

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
interface; its unresolved logical-vocabulary gap is not permission to weaken
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
hosts. Only selected modules and their prefixes enter namespace lookup;
adding an unselected module on disk must not silently add a declaration or
name collision. No implementation filename introduces another namespace.

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

Exercise check-only graphs without an entry, executable entry resolution and
no-heap enforcement over disconnected selected modules as well as the entry's
dependency closure. Ordinary allocating-prelude calls still reject under
no-heap, while unused implicit prelude availability does not. Working-directory
changes must not change root binding.
Canonical path aliases must not let one module appear twice in the graph.
An entry-only edit preserves unrelated source judgments; a no-heap policy edit
revalidates its affected type/call uses throughout the selected graph.

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

## Design suitability and remaining evidence

The selected architecture addresses the requested frontend and backend reuse
together. Its principal costs are complete dependency tracking, persistent
identity/evidence ownership, and LLVM planning integration. Those costs have
identified consumers: independent library edits, reusable generic instances
and optimized multi-module applications. A stable third-party artifact ABI,
remote cache service, new theorem language and incremental native linker do
not have a requirement here and are not selected.

The pending tree revision replaces the language root's closed-single-unit
decision with one project-root ordered adjacency file for all source-module
dependencies, including those used only by implementation. Earlier-target
checks certify acyclicity without coupling dependency direction to directories.
It replaces name-resolution's global inventory and top-level order dependence
with filesystem-qualified modules, direct directory ownership, shared declaration
names, file-local aliases and complete checked public interfaces. Root graph
rows declare the exact direct dependencies; row order certifies them without
adding unlisted edges or private access privileges.
The proposal also clarifies the compiler root's artifact boundary and adds one
compiler child for persistent incremental computation. The constitution's
safety and performance priorities, fixed deterministic proof families, generic
specialization and finite-cycle rule, callable contracts, source ownership,
backend fact obligations, self-tail semantics and runtime model remain in
force. The current generic scratch-analysis rejection is retained: different
verification contexts cannot share proofs merely because IDs are stable.

Four uncertainties are implementation acceptance work, not weaker endpoints:

- Verify interface/source grammars, exact declaration correspondence, public
  semantic closure, file-local alias formation, abstract representation/capability
  matching, order-independent formation and exact specification deltas. Qualify
  canonical root/path rules, direct directory membership and namespace/declaration collisions. Specify
  the root graph's complete ordered adjacency format, one canonical row per
  selected module, earlier-target checks and sole dependency authority.
  Qualify deep, parent/child and interleaved dependencies with ordinary privacy,
  complete interface-local alias qualification and no duplicate import lists.
  A separately compiled module private to a parent subtree remains a possible capability
  with no selected rule; reopen only for a concrete consumer that cannot use
  one module's private files. Qualify edge deletion, reordering and root binding;
  measure single-file coordination and query invalidation before claiming an
  edit-locality or collaboration advantage.
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
