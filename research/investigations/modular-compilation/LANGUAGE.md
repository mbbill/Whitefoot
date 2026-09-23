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
`public`, ordinary declaration indentation, `;` on a bodyless function,
`alias name = pkg::path;`. A graph row renders as
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
member path or callable signature.

All `.wfm` function items, public or private, are declarations ending in `;`.
Every one requires exactly one ordinary `.wf` definition. Definitions repeat
the entire signature and contract without `public`; ordinary private helpers
need no `.wfm` declaration. There are no executable bodies, generated property
bodies or trusted external source declarations in `.wfm`. Constants, complete
type schemas, groups and binding maps are declarative definitions.

Compare declaration and definition after resolving aliases and canonical
identities. Equality covers parameter order/names/modes/types, result
order/names/types, generic order/kinds/bounds, normalized
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

`public` permits executable field access; erased annotations also admit private
interface fields under the rules below. Ordinary ownership, effects, `readonly`,
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

## Runtime access and annotation visibility

Field privacy restricts executable source access. A field declared in a
complete `.wfm` representation may also be named in the following erased
annotation contexts, whether or not that field is `public`:

- `requires`, `ensures` and their contract-local `define` expressions;
- loop-header and local invariants, including relation premises in `use`;
- declared `reads` and `writes` paths, including function-kind signatures.

This rule applies both to the defining module and to other modules. It allows
an external wrapper, generic formal or explicit certificate to state the same
condition as an imported operation. Restricting hidden terms to the checker's
internal fact store would support some automatic call chains but leave those
written boundaries inexpressible. A module interface is self-contained because
its complete representations and the dependency interfaces contain every
structural component these annotations use.

The exception is for structural field/member selection, not general access to
private declarations. The root value and its type must already be
legally available in the writer's context. A path may traverse private supporting records in the
complete interface closure; their types are obtained from the preceding
field, not named as new public types. Ordinary type, variant-refinement,
initialization, domain, reference-validity and proof-expression restrictions
still apply at every step. A proof cannot inspect an uninitialized element or
an inactive enum payload simply because the path is erased.

Private top-level functions, constants and nominal names do not become public.
Public callable types, generic bounds and named constants used by its public
contracts must retain accessible vocabulary. Private constants needed only to
evaluate a representation or public constant remain allowed under interface
closure. No alias publishes private names or grants annotation privileges to
executable uses. Representations defined only in `.wf` remain inaccessible
outside their module; an annotation cannot make the interface read a body file.

Every field selector written by an external author still requires the direct
graph edge to that member's defining source module. Crossing into a dependency's
record may therefore require another explicit edge; prelude members keep their
ordinary availability. In contrast, consuming an already resolved callee
contract is a metadata traversal, not a newly written source reference. The
checker can carry its private-path facts without granting the caller new names
or graph permissions. The query engine records those metadata dependencies.

Executable field reads, writes, borrows, construction and consuming extraction
continue to require ordinary public access. An ordinary `let`, branch condition,
return expression or constant initializer is not an annotation merely because
its value will later help a proof. An annotation produces no runtime value,
borrow, memory read, instruction or effect, and cannot feed a value back into
executable code. The source role is fixed before resolution; later dead-code
elimination never changes which visibility rule applies. Diagnostic locations
and messages must distinguish inaccessible executable access from an unproved
or ill-formed annotation.

This deliberately permits representation-dependent proofs. Changing a private
field named by a public contract or client invariant can require client proof
edits and rechecking. Unrelated private fields do not become dependencies just
because their definition shares a file. Runtime encapsulation is preserved;
representation-independent specification vocabulary is not promised by privacy.

## Exact effects over private representation

Effects use their ordinary structural paths directly:

```text
public struct GrowVector<T, const ceiling: u64> {
  storage: Box<Slots<T>>;
  public tag: u64;
}

public fn len<T, const ceiling: u64>(values: &GrowVector<T, ceiling>)
  -> result: own u64 reads(values.storage.inner.len) contract {
  ensures result == deref(values).storage.inner.len;
};
```

An operation can declare `writes(values.storage)` while leaving `values.tag`
independent. A caller can repeat that row in a wrapper or function-kind formal,
but cannot execute a private `storage` selection. There are no named footprint
declarations, mapping expansion, extra disjointness axioms or new path syntax.
Ordinary field, enum-payload, prelude-window, parameter-index and range steps
retain their existing rules. Multiple substates use multiple ordinary row items.

Resolve each path to canonical member identities, then check declared versus
exhibited effects in both directions using EFF-2. An annotation mentioning a
field exhibits no runtime access and cannot justify a padded effect row.
Every declared effect still needs its ordinary body witness; `writes` retains
its existing coverage of reads. Aliases, prefixes, ranges and actual storage
identity decide overlap. Different spellings never imply independence.

`pure` retains its existing meaning: no state reads/writes and no promise of
termination. A runtime getter over a reference declares its actual reads. Proof
field mentions create support dependencies for fact validity but no runtime
effect or scheduling edge. Layout/member changes revalidate the precise path,
overlap, function-kind-refinement and fact-kill consumers that read them.

## Ordinary getters and private contract facts

There is no `observe` modifier, logical function call or `use view` step in
this proposal. An ordinary getter has a declaration in `.wfm`, a body in `.wf`,
and a written postcondition such as the `len` declaration above. Its body must
prove that postcondition under the ordinary checker. No automatic body-derived
equation is added to the caller's contract boundary.

After a real normal-returning call, the checker substitutes the selected
arguments/result into its verified postconditions, including private paths.
For example, `n = len(q)` establishes a relation between n and q's current
private length. A later branch `n < ceiling`, with no intervening overlapping
write, can discharge append's private-length requirement through the existing
integer proof rules. Caller executable source need not read the private field.
A call that never returns reaches no such continuation; getter termination is
not an added prerequisite. Getters may use ordinary control flow and arithmetic
subject to existing safety, effect and contract rules.

An erased contract cannot call this getter. It directly names the relevant
field path instead. This retains FN-8's exclusion of ordinary callable execution
and avoids introducing a second class of executable functions or a termination
checker. Calling a getter solely to obtain a proof fact is unnecessary when an
erased structural assertion can express the same obligation. When executable
code needs the value, it uses the ordinary getter and its normal codegen path;
no accessor-inlining or zero runtime cost is assumed without evidence.

## State, contracts, ownership transfer and proofs

A projected proof datum has the key
`(proof context, value/state image, canonical projection path, support versions)`.
Images are scoped to their body/instance and proof context, including separate
hypothetical refinement contexts; dense local IDs are not global identities.
Support includes the selected storage, holders needed to reach it, index/range
datums and live evidence needed for path formation. Aliased references resolve
to the same storage identity. Field privacy does not change that identity or
turn a mutable projection into a timeless value.

In a `requires`, a reference parameter's projected path denotes entry state.
In an `ensures`, its ordinary path denotes post-state immediately before return
transfer; `deref(entry(parameter)).field` selects its frozen entry image.
The explicit entry former retains its existing restriction to ensures and a
reference parameter whose selected path overlaps the enclosing function's
declared writes. Entry of a local or owned parameter, nested entry and runtime
entry remain rejected. A disjoint/read-only projection uses its bare path.

An owned parameter's contract path denotes its entry value even if the body
later consumes it. A named aggregate result's path, for example
`made.storage.len`, denotes the returned value before ownership transfer.
Extend the admitted FN-8/FN-9 operand forms to well-formed structural projections
from those owned input/result roots whose final type belongs to the existing
integer proof fragment. Retain the existing relation shapes and selected-result
rules: no arbitrary aggregate equality, new general enum result route or
proof-only borrow expression is introduced. Ordinary scalar result clauses
continue unchanged.

Each projection must satisfy the existing type, validity, refinement and
partial-operation-domain judgments in the relevant state before becoming a
datum. A requirement cannot assume itself to establish its own formation.
Frozen entry paths are formed from entry facts; returning-value paths are
checked against the actual returned value. Clause order supplies no circular
domain proof. Representation shape determines legal projections, never an
unstated relation between arbitrary fields.

Writes kill facts whose live support overlaps under ENT-5; disjoint writes
preserve them. Frozen entry datums are immutable mathematical values and do not
follow later writes. Capture only the projections referenced by the contract or
proof; no runtime snapshot or object copy is introduced. An old getter result
continues to exist as a scalar after a write, but its equality with the current
field no longer follows. Reference validity is checked independently: a retained
integer fact cannot make a stale borrowed holder valid again.

Construction, call-result binding and whole-owner transfer transport relevant
projection facts by structural substitution of the transferred value image.
Construction maps field values/measures into the constructed owner's image;
return maps that owner to the result ordinal; the caller maps the result to its
new destination. Move invalidates old live paths, then carries only relevant
facts whose affected support moves with that value and whose other support
remains live. Private fields participate exactly as public fields do.

This is not transport of the entire local proof context. Across calls, only
written verified relations and existing normative type facts are available.
An unrelated local fact or an implementation-inferred stronger theorem does not
cross the interface. Effect roots still belong to the current destination;
proof transport introduces no owned-value effect ancestry. References into the
old owner stay invalid. Copy creates a distinct storage identity with the copied
value facts; later writes separate the live states. Replacement kills the old
destination evidence. Multi-result transport is ordinal-local and failure-atomic.
The queue constructor therefore states `ensures made.storage.len == 0_u64;`
without a logical getter, hidden identity field or mandatory boxing.

There is no implicit source type invariant. Privacy alone establishes no
relation between fields. Public operations state their requirements and
guarantees; callbacks obey their ordinary boundaries even while an implementation
temporarily changes private state. A public mutable field linked to private
storage cannot rely on an unstated preserved invariant.

## Composition argument and its implementation obligations

This argument assumes the existing primitive, ownership and proof judgments;
it is a design argument, not a verification of an implementation.

First, annotation visibility adds names for already described structural
components. Formation still checks each term's type/domain/state. Naming a
private field establishes no fact and authorizes no runtime access. An erased
annotation has the same primitive meaning inside and outside the type's module.

Second, each body proves exactly its declared requirements/effects/guarantees
under the existing recursive-component restrictions. Exported clauses are
resolved expressions with member identities, not strings reparsed with the
caller's runtime privileges. Only normal-returning calls publish those verified
postconditions. A declaration or an unverified implementation supplies no axiom.

Third, substitution preserves the referenced state/value image. Actual writes
remove overlapping facts before postconditions describe the new state; entry
images remain frozen; checked ownership transfer renames only carried value
facts. Current proof availability and dependency equality are required when a
cached caller derivation is rebound to a changed implementation. Private-member
identity and meaning are dependencies when the consumed clause uses them.

Finally, composition validates every selected source obligation and current
dependency/availability edge before publication. Native symbol resolution is
insufficient. Cold/warm differential and mutation tests exercise these rules;
agreement between two executions of the checker is not a general soundness proof.

## Container, wrapper and explicit-proof qualification

The GrowVector boundary uses its existing algorithms and direct private paths:

```text
public fn append<T, const ceiling: u64>(values: &GrowVector<T, ceiling>, value: own T)
  -> length: own u64 writes(values.storage) contract {
  requires deref(values).storage.inner.len < ceiling;
  ensures length == deref(values).storage.inner.len;
  ensures deref(values).storage.inner.len == deref(entry(values)).storage.inner.len + 1_u64;
  ensures deref(values).storage.inner.cap >= deref(entry(values)).storage.inner.cap;
};
```

An external wrapper can repeat those clauses and the precise effect row without
executing a private selection. A function-kind formal can state the same complete
boundary; actual refinement and calls use ordinary FN-4/FN-5. A consuming
`free_empty(values: own GrowVector<T, ceiling>)` can require
`values.storage.inner.len == 0_u64` using its owned entry value. The queue demo
supplies the constructor/result-transport case and an external function-kind
consumer that writes private paths in both its `.wfm` and `.wf` contracts.

Use a caller loop with a header relation between its iteration counter and a
private length to qualify writer-visible erased access. Also require an
explicit multi-premise `use` certificate mentioning that length, beyond AUTO's
automatic family, so merely carrying hidden checker terms cannot pass this
criterion. Neither witness may insert runtime getter calls solely to name a
proof datum. Each written private selector must receive the same direct-edge,
type and state-formation checks as the equivalent public selector.

Positive/negative controls must distinguish the same private path in an
invariant from an executable `let`, branch, borrow, write, constructor or
destructure; only the erased uses get the visibility exception. Reject an
ordinary getter call in a contract, an unproved getter postcondition, use of
private top-level names, missing member-owner edges, uninitialized projections,
inactive payloads and stale references. Invalidate a saved live length relation
after append but preserve it across a tag-only write; keep its entry datum;
distinguish aliasing objects from independent ones; reject `free_empty` after a
nonempty transfer and declared effects without a body witness. Recheck consumed
private-path changes, but reuse source proofs after a getter body edit whose
written claims and current availability remain equal. These are implementation
acceptance obligations, not executed test results.


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
exclude erased proof annotations and uncalled declarations. Phantom type
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

[OpenJML's visibility explanation](https://www.openjml.org/tutorial/Visibility)
separates executable access from specification visibility and describes the
representation coupling of directly exposed specification fields. WF selects
one role-based rule for interface fields rather than another per-field
visibility modifier. This is a comparison, not adoption of JML's solver or
logical method calls. [Why3's type-invariant rules](https://why3.org/doc/syntaxref.html#record-types)
illustrate the extra construction and call-boundary obligations of implicit
invariants; WF retains explicit operation contracts.

The earlier finite `observe` / `use view` and named `footprint` candidate is
superseded: its premise that public annotations must hide every private path
is no longer selected. Direct erased paths cover the queue, wrapper, generic
formal, explicit-certificate and precise-effect requirements with existing
proof/effect families. Checker-only hidden facts were considered and rejected
as the complete authoring boundary because they leave external contracts and
manual invariants without names for required conditions.

Representation-independent model properties and mathematical functions remain
possible separate extensions. Reopen them for a concrete consumer whose
representation changes or algorithmic contract makes direct structural proofs
unsuitable; compare migration cost, totality/proof requirements and incremental
invalidation before choosing a new mechanism. No such abstraction is required
for this module implementation, and arbitrary runtime functions remain excluded
from erased proof expressions. No external reference defines WF acceptance.
