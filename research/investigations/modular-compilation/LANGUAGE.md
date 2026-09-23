# Module boundary rules selected for implementation

This companion to [DESIGN.md](DESIGN.md) closes its source-language choices.
It is a proposed amendment over this branch's active specification, not a
claim that the compiler implements these rules. [SYNTAX.md](SYNTAX.md) is the
complete grammar input. Existing rules apply unless a change is stated here.
Retain this document during implementation; fold its normative content into
the active specification and retire it when that specification owns the whole
boundary. The source demo remains an unexecuted design specimen.

## Formation and names

There is exactly one explicitly selected `modules.wfg` for a compilation.
Its directory is the package root; the process working directory is irrelevant.
There is no graph search, nested graph activation, external package binding,
version resolution, glob import, namespace declaration, or public re-export
through an alias in this design. External packages are deliberately outside
the user's selected scope, not an unresolved prerequisite for implementation.

A graph consists of one or more module rows followed by zero or more targets:

```text
pkg::data: [];
pkg::runtime::queue: [pkg::data];
pkg::runtime: [pkg::data, pkg::runtime::queue];
target tool { entry pkg::runtime::run; }
```

Each row registers one directory and lists its exact permitted direct
dependencies. Every dependency must already have appeared as a row. Reject
duplicate rows, duplicate edges, self-edges, missing rows, and a forward edge
at its written path. Strict position descent proves acyclicity; neither an
inferred topological order nor a cycle-repair heuristic chooses the architecture.
Reordering valid rows changes no declaration identity. Multiple valid orders
are allowed: canonicality does not impose alphabetical sorting of a DAG.

`pkg` alone registers the root module. Otherwise each lowercase identifier
after `pkg::` is a directory component. Registering a descendant does not
register its ancestors. Parent, child, sibling, and cross-branch modules have
identical permission and privacy rules. Naming a descendant does not require
edges to intermediate namespace directories. A direct edge to the actual
target is required even when a transitive path already exists.

Only `directory/module.wfm` and that directory's direct regular `*.wf` files
belong to a module. Child directories are not scanned into the parent.
Reject symlinked source roots, module directories, interfaces and source files,
path escape, duplicate physical ownership, and case-folding collisions rather
than selecting behavior by host filesystem order. Directory components use
the language's lowercase identifier alphabet. Source basenames are not symbols;
sort their relative byte spellings for deterministic diagnostics. An existing
unregistered directory grants no source visibility and is not silently built.

An explicit `alias Name = pkg::path::Name;` or
`alias short = pkg::path;` is legal only in the initial alias header of a
source/interface file. The right side is a complete `pkg`-qualified path,
never another alias, an instantiated type, or a call. Aliases are file-local
abbreviations preserving the target's identity and name domain. Uppercase
aliases denote types/groups/variants; lowercase aliases denote modules or
values/callables. Context checks reject the wrong domain. Reserve `pkg`;
reject duplicate aliases and ordinary prohibited shadowing, including an alias
that collides with a module-local declaration visible in the file. An unused
alias still receives formation, access and direct-edge checks.

There is one module-local inventory across `.wfm` and direct `.wf` records.
Declarations are forward-visible; local binders keep their lexical scopes.
Detect duplicate declarations independently of file enumeration. Resolve
constant dependencies, group dependencies and finite layouts separately;
name visibility does not admit their cycles. A same-module declaration moved
between `.wf` files retains identity but must resolve the new file's aliases.
Module path components and module-level lowercase declarations cannot occupy
the same qualified slot. Type-owned member labels are separate slots.
An unrooted qualified module path begins with an explicit file-local module
alias; directory proximity is not an implicit alias. Ordinary type-owned
group/variant qualification remains a distinct name domain.

Canonical rendering extends the existing source renderer: one space after
`public`/`observe`, ordinary declaration indentation, `;` on a bodyless function,
`alias name = pkg::path;`, and `footprint name = path;`. A graph row renders as
`pkg::path: [pkg::dependency, pkg::other];`, preserving written row and edge
order, with one row per line. Targets use ordinary two-space block indentation,
entry first and optional no_heap second. Semantic dependency sets normalize
independently of written edge order. No formatter infers an order or changes
the user's dependency architecture. Existing no-comment/doc rules remain.

Source-qualified references require an edge; compiler metadata traversals do
not grant one. For example, A may return B's public type and C may consume that
inferred value through A's API with only C -> A -> B. C cannot explicitly name
`pkg::b::T`, alias it, access a B member, or call B without C -> B. The compiler
still loads B's semantic/layout descriptions and records those actual query
dependencies. Passing an inferred value does not publish B's source namespace.

## Interface ownership and correspondence

Only `.wfm` accepts `public`; unmarked declarations and data members are
private to the module. There is no `private` keyword. `public` inside a private
type does not make that type or an access route public. Aliases, parameter
labels and local binders are never export declarations.

Every public source nominal has one complete definition in `.wfm`, including
its private fields and supporting declarations. Those definitions are reused
throughout the module. Reject a second, partial, or extending `.wf` definition.
A wholly implementation-private nominal may instead be defined once in `.wf`.
The `.wfm` must form using its own declarations, prelude and allowed dependency
interfaces; it never searches `.wf` for a missing type, constant, contract,
footprint mapping, or callable signature.

All `.wfm` function items, public or private, are declarations ending in `;`.
Every one requires exactly one ordinary `.wf` definition. Definitions repeat
the entire signature and contract without `public`; ordinary private helpers
need no `.wfm` declaration. There are no executable bodies, generated property
bodies or trusted external source declarations in `.wfm`. Constants, complete
type schemas, groups, binding maps and footprints are declarative definitions.

Compare declaration and definition after resolving aliases and canonical
identities. Equality covers parameter order/names/modes/types, result
order/names/types, generic order/kinds/bounds, the `observe` marker, normalized
effects, and ordered normalized requires/ensures. Alpha-normalize generic and
contract-local binders; retain public argument/result labels because callers
write them. Erased `define` sharing and equivalent resolved aliases may differ.
Use existing clause normalization; do not run a solver to excuse different
contracts. Reordering clauses is a change to the written boundary. A stronger
implementation requirement, weaker guarantee, extra effect, missing definition
or duplicate definition rejects. Internal invariants may prove additional
facts; those facts do not silently augment the public contract.

Interface formation can run before implementation checking. It produces a
well-formed claim, never permission to trust that claim. Current body evidence,
generic-instance checks and current proof-component availability are required
before any composed program or executable is published.

## Types, fields, variants, constants and groups

`public` permits naming a field; ordinary ownership, effects, `readonly`,
`opaque`, reference validity and release rules determine its operations.
External field construction requires every field accessible. There are no
default private values or hidden constructor writes. A factory is an ordinary
public function implemented inside the module. Whole-value ownership and
replacement remain ordinary operations and do not require private field access.

An extraction names only accessible fields. A consuming field move kills the
whole owner; no holes remain. Unbound parts, including private parts covered
by `..`, must have a legal ordinary release. A private linear remainder makes
that extraction illegal; consume it through an operation in the owning module.
Moved-owner references become invalid and later use rejects. Their mere
existence is not a new prohibition on the move. Copy fields are read without
`move`. `readonly` and source `opaque` retain their existing inside/outside
meaning; neither is a privacy synonym or a new construction escape.

Copy/drop capabilities are derived from every owning component, including
private support, under existing modifiers and generic substitution. `nocopy`
removes copy; `nodrop` removes both copy and drop. There is no handwritten
second capability declaration. Actual instance capabilities differ from what
a generic body's written bounds permit. Hidden linear ownership remains an
obligation; `nodrop` requires lawful explicit consumption, not a specially
named finalizer. No user-defined destructor or hidden runtime callback is added.

A public enum is a **closed case API**: every variant must be explicitly
`public`; leaving one private is a formation error. A private enum may have
unmarked variants. Payload members remain default-private and can be published
individually. Constructors require access to all their payload fields;
patterns bind accessible payloads and use the existing residual-release rule
for `..` (extended to enum arms by this proposal). Match coverage still includes
every variant, independently of payload access. There is no wildcard hiding a
case or unproved disposal of a private linear payload. An API that hides its
cases uses a public struct containing a private enum and ordinary operations.
This selects a closed sum rather than adding non-exhaustive enums and opaque
case matching to the module mechanism.

User enum constructors are type-owned: `Choice<T>::Some(value: x)` or
`pkg::data::Choice<T>::Some(value: x)`. A direct alias may abbreviate an
unspecialized accessible variant of a nongeneric enum; generic constructors
retain the explicit owner and its arguments. Pattern and result-route labels
remain unqualified and resolve against the known scrutinee/result type, not a
module-wide variant inventory. Compiler-prelude constructor spellings such as
`Ok<T,E>` retain their existing declarations. Type and enum member lookup
therefore need explicit owner identity in persistent keys.

A public constant has its exact type and initializer in `.wfm`. Its value,
including private helper-constant evaluation, must be determined there.
Published types/bounds/contracts cannot require clients to name a private
constant or nominal. Private constants used only in private field types are
allowed. Const evaluation and eligibility remain unchanged except that the
acyclic dependency graph replaces source declaration order. Public constants
are compile-time values, not dynamically initialized globals. Modules introduce
no initialization order or initialization functions.

A public `interface` publishes the complete ordered function-parameter group.
Its member labels are formal parameters, not individually exported functions,
and have no visibility modifiers. A public `binding` publishes the complete
actual bundle with its checked group boundary. Private actual functions must
have complete private declarations in the same `.wfm` so the map is
self-contained; publication permits use through the bundle, not direct
source naming of those actuals. Ordinary FN-4 refinement and FN-5 authoritative
formal calls still apply; no runtime dictionary or implicit implementation is
introduced. A public bundle is an explicit API choice in the sole interface,
unlike an alias that cannot publish anything.

## Named footprints over private representation

A footprint is a declarative type member, usable only in effect paths:

```text
public struct GrowVector<T, const ceiling: u64> {
  storage: Box<Slots<T>>;
  public tag: u64;
  public footprint state = storage;
  public footprint length = storage.inner.len;
  public footprint capacity = storage.inner.cap;
  public footprint elements = storage.inner;
}
```

This adds no field, storage, executable getter, reference, capability, or proof
axiom. `writes(values.state)` expands to `writes(values.storage)`;
`reads(values.length)` expands to its length measure path. An operation that
updates storage can remain independent of `values.tag`. Two different
footprint names are **not** assumed disjoint. Their expanded paths decide.

Formation resolves one relative structural path from this
nominal's complete definition. Names share the field-label namespace and
cannot collide with fields. Paths may traverse fields, prelude measures/window
parts and enum payload steps. The final step may name an already formed
footprint, including a public footprint of a dependency. Expand this acyclic
single-successor chain; reject a cycle. One mapping refers to at most one other
mapping and cannot duplicate its expansion. No union, value expression, function call,
subscript, heap walk, object set, ghost state or recursive footprint definition
is admitted in the declaration. Several independently named substates use
several ordinary row entries; one union name is unnecessary for that precision.

A concrete suffix after a footprint in an effect path must be well-typed for
its expanded endpoint, retaining ordinary member-access checks for written
suffixes. Ordinary argument-supplied indices and ranges therefore
remain available on a footprint denoting a storage window. Expanded paths use the
ordinary alias, prefix and range-overlap judgment. Check the ordinary duplicate
and category/order rules after expansion too; two synonymous entries cannot
evade the existing duplicate-path rejection. Internal prefix sharing does not
silently repair an invalid written row.

Check declared versus exhibited effects in both directions after expansion,
with the existing EFF-2 coverage rule. Every expanded declared path needs its
ordinary exhibited witness. `writes` keeps its existing coverage of reads. Only source access
to the published footprint is authorized, not source access to its private
path. The compiler may use that path for checking and optimization.
Changing a footprint map revalidates consumers of its meaning, especially
disjointness, observer support, function-kind refinement and proof kills.

## Integer observations and executable getters

`observe fn` selects a **finite scalar view**, not arbitrary execution in logic:

```text
public observe fn len<T, const ceiling: u64>(values: &GrowVector<T, ceiling>)
  -> result: own u64 reads(values.length);
```

Its ordinary runtime definition lives in `.wf`. There is one callable identity,
one signature, and one implementation; logical use is erased. Its `.wfm`
declaration may state additional ordinary requires/ensures. Calls of an
unmarked getter remain ordinary calls and cannot occur in proof expressions.

The following admission is the selected complete rule, not a temporary trust
path awaiting a general termination checker:

1. One `own` result of an ENT-2 fragment integer type. Parameters are references
   to ordinary values/windows or owned fragment integers. Type/const parameters
   and explicitly `observe` function formals are allowed. No owning aggregate
   input, returned reference, multiple result, float or Boolean observation.
2. The implementation is a straight-line sequence of ordinary immutable `let`
   bindings, local invariants and one return, optionally with its ordinary doc.
   Its values are integer literals/constants, supported scalar projections,
   or calls to admitted observations. Arithmetic is exact addition/subtraction
   and multiplication by a compile-time integer constant, plus integer
   conversions proved to preserve the mathematical value, producing an affine
   DAG. A multiplier must be a constant in the current checking context;
   an unresolved generic value is not silently treated as a known coefficient.
   Ordinary range/overflow/domain and access obligations must all prove.
   Non-affine arithmetic, branches, loops, recursion, `set`, consumption,
   allocation, release, external operations, and calls to other ordinary
   functions are outside this marked form, even when dead or optimized away.
   The admitted arithmetic/conversion operation-table rows are typed affine
   nodes, not a permission to call arbitrary prelude functions.
3. Concrete observation dependencies from bodies and domains form an acyclic
   graph. An observer's requirements may use predecessor observations only
   when those predecessors have no requirements of their own. This permits
   an indexed observation guarded by a total len, without recursively
   generating a tree of domain obligations at each logical use. No observer
   refers to itself or an admission that depends back on it.
   Check generic schemas and each demanded concrete binding using the existing
   finite-instantiation discipline. A function-formal actual must carry a
   successfully checked observation realization as well as ordinary refinement.
4. The normal body checker establishes exact read-only effects and every
   declared guarantee without assuming its own guarantee. Separately, a linear
   scan substitutes local SSA bindings into a shared affine view DAG. Its
   nodes refer to parameter ordinals, canonical constants, private scalar
   projection paths and predecessor observation identities. Arithmetic nodes
   carry the ordinary discharged domain evidence. No new trusted function
   declaration or unchecked equation is accepted.

The acyclic finite expression and ordinary domain proofs establish a unique
terminating integer result whenever the declared domain holds. Extraction is
not general symbolic execution: no branch paths, loop iterations, recursive
unfolding or runtime values are enumerated. A complicated runtime query can
remain an ordinary getter; making every such query a logical term is not a
requirement of a module boundary. General recursive mathematical functions
would be a separate language decision with termination and proof-cost evidence.
Builtin storage measures remain their existing field reads; no second builtin
len_of/cap_of reader is introduced. A source observer defines the API of
its own abstract type, with its body reading ordinary fields.

At external boundaries the observation is opaque: callers use its identity,
signature, exact support, integer range and checked guarantees. They do not
receive its private equation. Inside its defining module, a view that is just
one scalar projection, after eliminating local copies, is canonicalized to
that projection at each source-written observation occurrence. That fixed
automatic family handles `len`/`cap` and does not recursively unfold calls.

For a computed affine view, an explicit step in a local invariant can request
one checked expansion: `use view remaining(values: values);`. This replaces
that application in the current certificate's goal/premises by its one-layer
affine realization, then uses the ordinary affine certificate fold. Nested
observations remain terms; each further expansion requires a written step.
The application must occur in that certificate, its domain must already hold,
and its body must belong to the checking module. Repeated, unnecessary or
inaccessible expansions reject under ordinary certificate redundancy/access
principles. The step changes no surrounding fact state except the invariant's
proved conclusion. Its provenance includes the current realization receipt.
No equality axiom, automatic recursive unfolding or solver search is added.

Automatic work registers only applications written in source, runtime-call
bridges, and those introduced by explicit one-layer steps. Congruence uses
canonical identity and equal normalized argument/state keys; it generates no
new applications. Graph formation, shared affine extraction and each explicit
expansion are polynomial in declarations, materialized applications and written
proof size, with integer bit complexity included. Existing automatic families
run to completion on that finite term set; cache state and time limits never
select acceptance. There is no attempt to derive arbitrary properties of an
uninterpreted function.

## State, contracts, ownership transfer and proofs

An observation application has the key
`(proof context, instance, scalar input datums, argument view identities, support versions)`.
Images are scoped to their body/instance and proof context, including separate
hypothetical refinement contexts; dense local IDs are not global identities.
Proof support includes resolved read paths, the support of domain requirements,
the holders needed to reach them, and index/range datums. A logical domain read
adds proof support but no runtime effect. Even a constant-result observer with
a mutable-state requirement cannot leave a live, well-defined application
behind after that requirement's support is killed. Aliased references resolve
to the same storage identity.
A bare reference argument observes its current state. `entry(parameter)`
selects its entry image; it never follows later mutation or reference rebinding.

In an `ensures`, a current reference view denotes the post-state immediately
before the selected return transfer; in `requires` it denotes the entry state.
An owned parameter's proof view always denotes its entry value, including
`&parameter` inside contracts, so consuming it in the body does not lose the
written input relation. A result's proof view denotes the returned value
before ownership transfer. `&made` in `len(values: &made)` is a proof-only
view constructor in a contract, not an escaping reference or runtime borrow.
The return/call mode and ordinary reference restrictions are unchanged.
The explicit entry former remains restricted to ensures and a reference
parameter written by the enclosing function. For an observation, at least one
of that argument's observation-support paths must overlap the expanded write
row. A disjoint/read-only observation uses its bare view. Entry of a local or
owned parameter, nested entry, and runtime entry remain rejected. Owned input
views already have their entry meaning without the former.

Formation checks types, place validity, public access, domain requirements and
non-consuming argument shapes. It does not execute a getter. Each occurrence
must establish its observer's requirements from the entering context; a clause
cannot establish the domain needed to make itself well-formed. `entry` domain
obligations use entry facts, not later postconditions. Clause order does not
permit circular domain discharge. Return-view formation also checks the
actual returned value and its private layout where relevant.

Every observation denotes one integer datum. FN-8 retains its existing clause
judgments with this additional operand; FN-9 retains its difference-bound
relation shape with observations as datums. A named aggregate result can now
supply an observation datum even though the aggregate is not itself an integer.
This is not arbitrary aggregate equality or admission of every nested result
projection. Existing routed-result restrictions remain; general enum result
routes are not silently introduced with modules.

Writes kill facts supported by overlapping live views under ENT-5. A disjoint
write preserves them. An entry observation is an immutable mathematical datum
with no live storage support after capture; only the observations referenced
by the contract/proof are captured. No runtime snapshot or object copy exists.
After mutation, a new current observation is distinct from the old one; it is
never equated merely because the argument still has the same source name.
Reference validity is checked independently, so a stale path cannot form a new
observation even if some old integer fact survives.

A runtime observation call returns the same value as the corresponding
pre-call view, with its domain proved and read support captured at that call.
Its normal result binding receives that equality as a generated checked
boundary relation, in addition to written ensures. The relation's authority is
the independently checked view realization, not an assumed self-postcondition.
If later mutation kills a relation to a *live* view, the scalar result value
still exists; it does not magically become the new getter result.

Aggregate construction, call-result binding and whole-owner transfer transport
proof views by structural substitution of the transferred value image.
Construction maps private field measures into the constructed owner's view;
return maps that owner to the declared result ordinal; a caller binds the
result view to its new destination. Move invalidates old live paths, then
transports only the relevant scalar measure/observation facts and structural
construction/copy equalities whose affected supports are carried by that value
and whose other supports remain live. This is not a transfer of the complete
local proof context. Across calls, only declared relations and the checked
observation-call bridge are published. Effect roots still belong to the
current destination; proof transport introduces no owned-value effect ancestry.
References into the old owner remain invalid; value-view transport never
revalidates them. Copy transports a value image to a distinct storage identity;
future writes separate their live views. Replacement kills the destination's
old views before binding the new ones. Multi-result transport is ordinal-local
and failure-atomic. No arbitrary old local, unrelated heap fact, or killed
support is resurrected. Constructor/factory postconditions therefore apply to
ordinary by-value structs without boxing or a hidden identity field.

There is no implicit source type invariant. Privacy alone establishes no
relation between fields. Public operations write the requirements and guarantees
they actually prove, and callbacks must satisfy their ordinary boundaries even
when an implementation temporarily changes private state. A public mutable
field tied to private storage cannot rely on an unstated preserved invariant.
This retains the existing proof model instead of introducing a second
construction/callback/invariant protocol as part of modules.

## Composition argument and its implementation obligations

The argument relies on the existing primitive operation, ownership and proof
judgments; it is a design argument, not verification of an implementation.

First, type/interface formation and footprint expansion add only checked names
for the same nominal values and structural states. Their access restrictions
remove source access; they neither invent values nor assume disjointness.
Second, induction over the observation dependency DAG gives each admitted
observer a total affine interpretation on its domain: leaves are valid scalar
reads/constants, arithmetic has domain evidence, and predecessor calls have
checked domains. Domain-free predecessors keep logical domain formation from
recursively unfolding requirements. Runtime extraction and logical view refer
to the same typed operations; contracts cannot justify their own realization.

Third, each state transition either preserves disjoint support, kills changed
live support, captures an immutable pre-state datum, or transports an actually
transferred value image. These are separate from reference validity and effect
root attribution. The constructor, copy, move, call and return cases need
explicit implementation tests and proof-fragment parents; merely matching a
view's printed name is insufficient.

Fourth, ordinary postconditions publish only after their current concrete
proof component succeeds, using no same-component postcondition assumptions.
An external derivation is parametric in the opaque observation interpretation:
it uses only the admitted domain/support and checked claims. A new realization
can replace an old one when those consumed claims remain identical and have
current evidence. A derivation that expanded the old realization is not
parametric in it and must revalidate that dependency. This is the substitution
ground for claim rebinding, not trust in unchanged `.wfm` bytes.

Finally, the composed receipt validates every selected source obligation and
current dependency/availability edge before publication. Native symbol
resolution and successful linking cannot substitute for those premises.
Differential cold/warm and mutation tests exercise the transitions and receipt
bindings; they complement this argument rather than proving general soundness
by agreement of two executions of the same checker.

## Abstract-container qualification witness

The selected boundary covers the existing GrowVector design without exposing
`storage` to caller source. Its relevant declarations are these fragments;
their omitted implementations are the existing algorithms to migrate, not
code claimed executable by this document:

```text
public observe fn len<T, const ceiling: u64>(values: &GrowVector<T, ceiling>)
  -> result: own u64 reads(values.length);
public observe fn cap<T, const ceiling: u64>(values: &GrowVector<T, ceiling>)
  -> result: own u64 reads(values.capacity);
public fn append<T, const ceiling: u64>(values: &GrowVector<T, ceiling>, value: own T)
  -> length: own u64 writes(values.state) contract {
  requires len::<T, ceiling>(values: values) < ceiling;
  ensures length == len::<T, ceiling>(values: values);
  ensures len::<T, ceiling>(values: values) == len::<T, ceiling>(values: entry(values)) + 1_u64;
  ensures cap::<T, ceiling>(values: values) >= cap::<T, ceiling>(values: entry(values));
};
```

`len` and `cap` implementations return `deref(values).storage.inner.len/cap`.
Their checked projection realizations translate the implementation's view
clauses to the current library's measure clauses. An external wrapper repeats
the public clauses and calls `append`; it uses the checked summary without
private unfolding. A function-kind formal repeats those same effects/clauses;
the supplied append function is checked independently and calls use the formal
boundary. `free_empty(values: own GrowVector<T, ceiling>)` can require
`len::<T, ceiling>(values: &values) == 0_u64` using an owned entry view.
The queue specimen supplies the corresponding constructor and aggregate result
case with actual proposed source bodies.

Required controls: reject a getter returning a different field while claiming
an unproved bound; reject an unmarked getter, observation cycle or non-affine
body in logic; invalidate a saved live length fact after append; preserve it
across a tag-only write; distinguish two aliases of the same object from two
independent objects; reject a stale borrowed holder; preserve the frozen entry
length; reject `free_empty` after a nonempty transfer; reject a padded footprint;
and recheck a private footprint-map change that creates overlap. These are
semantic acceptance obligations for implementation, not passing test claims.

## Target requirements

A named target selects exactly one public ordinary entry function by a full
`pkg` path. It does not change FN-7/PROG-3: the build must establish a complete
ordinary call binding, including explicit generic actuals, argument types,
ownership and requirements, and ordinary result handling. That checked binding
is a target-composition input, not permission to assume requirements from a
command-line value. The graph does not contain runtime values or invent a
second argument language. The native launcher initially retains its current
supported binding shapes, using the selected identity instead of a hardcoded
name; a nongeneric wrapper is the direct source route to a different instance
or argument adaptation. Unsupported launcher binding is a compiler capability
diagnostic, not rejection of an otherwise valid source function or graph.
Duplicate targets and missing/private/wrong-kind entries reject. A graph may
have no targets for check-only use. Check-only formation does not execute an
entry or demand that a native launcher support every valid callable signature.

All graph rows are structurally checked for every invocation. A target checks
every source definition/template in its entry module's declared dependency
closure, not just called bodies. Check-only may select one module closure or
all graph modules. Required concrete instances still receive ordinary checking.

`no_heap` is a target requirement over the conservative concrete execution
closure, seeded by the entry and platform startup. Include all syntactic calls
in selected concrete bodies, called function-formal actuals, materialized
parameter/local/result/value layouts, derived release and required native/
runtime supplies. Include calls in branches regardless of constant conditions;
exclude erased proof observations and uncalled declarations. Phantom type
arguments contribute only through their actual value/layout/release uses.
An allocation, required heap-bearing value, heap release or runtime heap need
rejects that target. Private representation is not an exemption. Fixed-point
summaries retract on edge/body deletion and are evaluated before optimization.

An unused allocating helper in a selected source module remains fully checked
but does not impose a heap need on this target. Objects and runtime selection
must likewise exclude its allocator dependency. This deliberately replaces
STOR-8's compilation-unit spelling ban; merely moving `program no_heap` into
the graph without changing its judgment is insufficient. Ordinary module
proofs are shared between heap-enabled and no-heap targets.

## Sources and rejected extensions

[Dafny's reference](https://dafny.org/latest/DafnyRef/DafnyRef) distinguishes
function domains, read frames and termination; that comparison explains why
read-only is not sufficient for an observation. This proposal instead selects
a finite affine view and Whitefoot's existing proof families, without Dafny's
solver or general recursive functions. [Why3's type-invariant rules](https://why3.org/doc/syntaxref.html#record-types)
illustrate that implicit invariants need construction and call-boundary
obligations. This proposal retains explicit operation contracts. Neither
reference is an acceptance authority for Whitefoot.
