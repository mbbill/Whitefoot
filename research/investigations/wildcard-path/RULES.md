# Proposed wildcard-path rules

This is a research amendment to kernel v0.60, not an amendment of the active
specification. [REPORT.md](REPORT.md) gives the selection grounds and limits;
[CASES.md](CASES.md) gives the discriminating programs. Rule labels below are
local draft labels. No source token, grammar production, reference kind,
effect-row spelling, or runtime representation is added.

The proposal needs more than a replacement of REF-1's static-shape sentence.
In particular, REF-2, ENT-3.S15, EFF-2/5, OP-11, and the consumers of resolved
places must use the distinctions below. Keeping their current text literally
unchanged is not this proposal.

## Paths, targets, and formation

[WP-1] A widened reference is a local name for one existing place of its
referent type. Its path description is an approximation of that place, not a
name for every place in that approximation.

Write `cone(R)` or `R.**` for the set consisting of R and all its finite owned
descendants, including descendants reached through Box content, fields,
currently present variant payloads, and initialized elements. The empty
suffix is included. These are notation in the checker and this specification,
not writer syntax. The reference retains its exact referent type; widening
neither converts a type nor makes a nonexistent payload or element selectable.

R is an evaluated, resolved place description, including captured index and
range values, not a reference variable whose later rebinding retargets R.
A cone retains its ultimate root and that root's scope. Moving the root does
not reroot the cone. A description may contain alternatives with different
roots; no alternative is discarded because another is convenient to check.
An anchor approximated by a containing prefix denotes the larger cone.

The checker distinguishes three things:

1. The cone or cones containing the target, used for possible interference.
2. The current target identity, used for equal-place and value-fact reasoning.
3. The validity and writability judgments for that target.

Equal cone descriptions establish neither equal targets nor distinct targets.
Two occurrences of `deref(p)` before any rebinding denote the same target.
`let q = p;` snapshots that target; a subsequent rebinding of p does not
retarget q. A checked projection from a target retains its finite relative
suffix. Identity and relative-suffix information are retained only where
established by formation, aliasing, or a join that proves them on every edge.
There is no equality test on raw pointers and no general reachability solver.

[WP-2] Forming a reference checks the complete selected place under the
ordinary type, ownership, readonly, refinement, index, and range judgments.
Extending a widened reference checks its source reference's validity first
and each newly written selection under the current facts. `R.**` supplies no
variant fact, numeric bound, or permission to perform a partial operation.

A reference definition selected for widening under WP-6 packages the checked
existence of its selected place as an ownership-side witness. Its meaning is:
there exists a finite valid ownership path, under one of the recorded
anchors, to the captured place of the stated type. The witness contains no
runtime path, length, tag, region, or counter. It is checked by induction over
formation and the transfers below, not by enumerating the path at runtime.

Variant and bounds evidence for the already traversed, hidden part of this
path are sealed into this witness. Merely leaving the source match arm or
forgetting a source-level bound does not revoke the witness. It grants no
right to spell that old payload or index selection again. A newly written
`.Some.value` still needs its own current refinement. A newly written
subscript still needs its own current bound. Ordinary, unwidened references
retain REF-2/4's existing refinement and bounds treatment.

This sealing is an explicit change to REF-2 and ENT-3.S15 for widened
references. It is necessary for the backedge of CASES C01. It is not a new
ENT-3 fact source and does not publish an existential proposition into ENT-1.
WP-4 protects the hidden discriminants, initialized extents, Box owners, and
storage against subsequent destructive changes.

The recorded write permission is the conjunction of the permissions of all
possible formation paths, including every readonly prefix and every formal
row restriction. Widening cannot turn a read-only path into a writable one.
TYPE-8, REF-3, linearity, and the prohibition of holes are unchanged.

## Overlap and proper prefixes

[WP-3] A cone is an interference cover, not an ordinary field step.
An ordinary path P is disjoint from `R.**` only if OWN-7 proves P disjoint
from R before entering the unknown suffix. Thus an ancestor of R overlaps
the cone, as does R itself and every descendant of R. Two cones are disjoint
only if their anchors are proved disjoint by that same judgment. Alternatives
are compared universally. An unresolved anchor proves no separation.

Captured indices and ranges in the known prefix use OWN-7, including WIN-2's
window-part relation. A differing field or index *after* two unrelated
unknown targets does not establish separation. In particular, different
spellings in `deref(p).left` and `deref(q).right` establish no separation when
p and q independently name descendants of the same anchor.

There is one bounded precision rule. When both accesses are relative to the
same captured target identity, compare their finite suffixes by ordinary
OWN-7. For example, `deref(p).tag` and `deref(p).next` are distinct fields of
one existing Node even when p is widened. The same comparison applies to a
proved alias of that current target. Merely sharing a cone cannot use this
rule. This rule is uniform for calls, validity, fact kills, and PAR.

Equal-place, may-overlap, and may-be-a-proper-prefix are different judgments.
A cone does not establish that its target is the anchor, an ancestor of a
second target, a descendant of that second target, or a different target.
When no established target relation or known-prefix separation decides a
potentially destructive prefix relation, treat it as possible and invalidate
under WP-4. Never infer a negative prefix relation from absence of the hidden
steps in the abstract representation.

## Invalidation and facts

[WP-4] A widened reference is valid only while its WP-2 witness is retained.
The following rules apply to the full semantic action, including projected
call effects, assignment disposition, an atomic update, window mutation,
consumption, and scope-exit release. A compound action is checked as one
batch against every affected reference. Protection from one member of the
batch does not protect from another member.

Root death, root scope exit, and destruction or relocation of any possible
proper prefix invalidate the reference. An event at an ancestor of R is
included; it need not be spelled below R. A partial consuming use is judged
with OWN-1/WIN-3's whole-owner consequence, not merely the source sub-place.
Invalidation does not forbid the mutation. A subsequent use without a new
valid formation is the REF-2 rejection. Replacing or moving back a value does
not revive an old witness.

A content-preserving action on an existing target leaves that target in
place and initialized on normal continuation. Ordinary assignment, OP-11
exchange satisfying WP-8, and OP-12 atomic update have this property for
their targets. A raw consume or free does not acquire it by being spelled
through a reference.

For a structural write, use the following closed preservation tests:

- A proved disjoint access preserves the reference.
- A content-preserving action proved to be at or below the reference's
  captured target preserves the target witness. This includes writes through
  that reference, through a proved equal-target alias, and through a checked
  finite projection of it. It does not preserve descendants of the written
  place or ordinary facts about its old value.
- An ordinary reference proved to name an ancestor of every possible written
  target is subject to ordinary REF-2's content-write rule. Losing its own
  unsealed refinement or bound still invalidates it.
- Every other potentially overlapping structural write invalidates a widened
  reference. It invalidates an ordinary descendant reference whenever the
  write might remove a proper prefix or required selection support of it.

For example, a write through p leaves p valid and invalidates an unrelated
widened q in the same cone. A proved alias of p can survive that content
write. A reference to a child destroyed by the write cannot. This improves
the proposal's unconditional invalidation of every other reference without
introducing an alias search.

An ordinary write to an existing TYPE-1 primitive leaf preserves widened
target witnesses even when its place may overlap their cones: it replaces no
owner, discriminant, storage allocation, or initialized extent. This test is
on the semantic write and its selected type. It excludes compiler-owned
window measures and parts, window boundary operations, moves, frees, and
actions whose declared write covers an aggregate. A primitive field named
`len` in a source struct is an ordinary field; a window's `len` is not.
The minimal rule deliberately does not broaden this exception to every copy
type. All other writability and readonly checks still apply.

Window actions retain OP-10's selection consequences. In particular,
removing a possibly containing slot, invalidating its required initialized
extent, remaking a containing allocation, or shifting a Ring's logical
coordinates invalidates the witness. No proof about a captured logical
index allows its address to survive such a change. An external append within
an overlapping coarse cone may conservatively invalidate a cursor even
when no existing element moved; the ordinary known-prefix disjoint case
remains available. Sealing removes a lexical fact dependency, not these
storage dependencies.

[WP-5] Validity is not a value fact. ENT-5's support kills apply to every
write, including a primitive leaf write and a content write that preserves
the reference itself. Overlap of a support with a widened access is WP-3's
overlap. Refining a current enum still uses ENT-3.S15 and loses that exposed
refinement on the ordinary events; a hidden path witness is not that fact.

A rebinding replaces the current target identity. It kills every current
place fact, comparison origin, signed goal, exposed variant refinement, and
entry-image stability assertion that depends on reading through the old
binding. An unchanged anchor does not preserve any such fact. Facts on a
different unchanged alias can survive where their own support survives.
An immutable value image or invariant theorem about a past value remains a
theorem about that value, never a theorem about the new referent.

ENT-2's term eligibility and identity are unchanged: the resolved
declaration event and canonical source spelling identify a tracked term;
different spellings remain different terms even when they alias. Each
term's storage support additionally refers to the current captured target
and its typed suffix for kill judgments. Neither the string `R.**.field`
nor a proved alias replaces the existing source-term identity. The
ordinary source `deref(p).field` can be reused after rebinding only after
the old supported facts have been killed. A loop-header target identity
denotes the arbitrary iteration being checked, not equality to a previous
iteration's referent. The final fact walk uses ENT-2's existing one-abstract-
evaluation convention; it does not mint a term for each dynamic iteration.

FN-8 requirements and FN-9/CALL-6 postconditions keep their v0.60 formation,
entry/exit, route, support, and publication rules. Substitute the ordinary
actual Goal/term identities for formal datums and use their captured targets
for support and overlap. Apply all write and rebinding kills before
publishing only the relations those rules admit. In particular, this change
does not admit a new scalar-field exit postcondition, invent a snapshot, or
transfer a contract about one cursor to another cursor in the same cone.

## Joins and loop convergence

[WP-6] Replace REF-1's prohibition on self-extending loop-carried paths with
the following finite summary construction. Static-shape reference paths
that do not participate in growing reference flow retain their existing
judgment. No runtime-depth limit is introduced.

Build the finite graph of reference definitions, rebindings, and joins in a
function's structural control graph. A reference-flow edge carries the
finite selector suffix appended by that source occurrence; a reset from an
owned place is a seed. Ordinary aliases have empty suffixes. A cycle with a
nonempty selector suffix is a growing component. Detect cycles through
aliases and several variables as well as a direct self-reference. A growing
component in a nested loop is not a new owner or a new runtime anchor.

For summary construction only, propagate one containing prefix per ultimate
root through the growing component and its summarized dependents. An entry
path R starts as R. A descendant-producing cycle replaces its varying tail
by `R.**`; every subsequent descent within it is absorbed by the same cone.
For paths of the same root not contained by the current cone, shorten its
anchor to their longest common definitely identical prefix. Different roots
are retained as different alternatives. A reset to an ancestor can therefore
shorten the anchor; a reset to another root adds that root. An already
summarized input contributes its existing anchor, not a new anchor named by
the current cursor. Normalization admits no nested `**` and no suffix after
`**` in a *cover*. Finite relative suffixes for a current target under WP-1
are separate from that cover.

For this prefix computation, identical selectors mean identical fields or
payload steps and identical immutable captured index/range values. When a
selector's values differ, are not known identical, or vary between
iterations, stop the common prefix before that selector. Do not enumerate
its possible integers or expand successive arithmetic recurrences. A
formation within an iteration may still retain its one captured index for
ordinary local proofs. Captures from repeated evaluations of one source
site are not equal merely because their source coordinates are equal.

Process components in dependency order. Within a component, visit equations
in source NodePath order, joining the entering states and all structural
backedge contributions, until no cover changes. A loop's entry R is not
replaced by its most recently discovered descendant. At a summary join,
validity holds only when every reaching edge carries a valid witness and
writability holds only when every edge permits writing. Equal-target and
finite relative-target relations survive only when all edges establish the
same relation. An invalid alternative never disappears by being covered by
a valid cone. These are independent of numeric contradiction elimination.

Compute target covers before the final body judgments and effect projection.
Recheck all uses, calls, support kills, refinements, bounds, and exiting
edges against the settled covers. Reference validity and retained
equal-target information are finite must analyses: joins can lose a must
property, and only a checked formation at its source node can establish a
new local witness. Do not iterate ENT-5 numeric bounds around the loop.
Use ENT-5's specified conservative loop fact state and INV-1's explicit
base and preservation checks after the covers settle. No body-established
numeric fact becomes an invented loop invariant.

Break and counted false-header edges keep their own post-event states.
Loop continuations join those states as ENT-5 prescribes, retaining the
target cover and validity information. Neither leaving the loop nor
entering an outer loop restores the entry target. A fresh reset can obtain
an exact target in straight-line code; it does not narrow a previously
settled loop-header cover. Scope cleanup precedes all joins. OWN-11/LIV-1's
structural liveness agreement remains an independent prerequisite.

This procedure runs to completion. There is no prescribed single recheck,
timeout, iteration fuel, machine-speed limit, or cumulative work budget that
selects acceptance. CASES C28 and C29 distinguish these requirements.

## Calls, exchange, and execution footprints

[WP-7] EFF-5 substitution preserves the actual's captured target identity,
its possible anchors, and the finite suffix of each declared row entry.
It does not convert that target into its anchor and does not forget the
callee's suffix before checking same-target relations under WP-3.
All index and range arguments keep EFF-5's evaluation-once meaning.

Pairwise comparison still includes all distinct substituted row entries and
by-value accesses. Independent actuals whose cones overlap and whose
effects include a write are rejected unless WP-3 proves their accesses
disjoint. Read/read overlap is admitted. The comparison precedes every
post-call invalidation: invalidating one cursor afterwards cannot justify
interference during a call. The same-target suffix test allows one cursor
actual at a row such as `reads(node.tag), writes(node.next)`.

Run WP-4 on every live caller reference, including actual arguments. An
argument is protected from a content write at or below its own captured
target, not from another actual's write that can destroy its target. Thus
an unused cursor argument can become invalid after a permitted call which
changes its ancestor. The callee's row alone determines this consequence;
no body inspection or effect inferred from an actual's surface spelling
is allowed. A callee's local rebinding of a parameter does not rebind the
caller's reference variable.

Source functions remain subject to ordinary ownership: receiving a
reference does not grant ownership of its referent, permission to create a
hole, or permission to destroy the caller's target on normal return. A
whole-target write may replace its value and destroy descendants, but
leaves that target initialized. No new per-function lifetime summary is
assumed in order to establish this property.

For EFF-2, an access through a summarized target rooted in a reference
parameter contributes the nearest ordinary effect path that covers its
anchor and unknown descendants. If no narrower EFF-1 path can name the
anchor, contribute the enclosing formal path. The row grammar remains
unchanged: a walker can declare `writes(root)` but cannot declare
`writes(root.**)`. Both directions of EFF-2's exhibition check remain;
an access below the declared path exhibits it as v0.60 already permits.
Local-rooted effects still frame out of the enclosing signature and remain
in the local checked footprints.

[WP-8] OP-11's equality allowance is not an allowance for a possible
proper-prefix relation. For every possible pair of exchange targets prove
that they are equal or disjoint, with neither a proper prefix of the other.
A same-target identity proves equality. Two possibly equal indices in the
same array or window at the same complete slot depth are equal-or-disjoint
by the ordinary storage rule, so existing index swaps need no disequality
branch. A pair which could be ancestor and descendant is refused even
though it has one type and even though both operands are references.

Apply WP-4 simultaneously to both writes of an admitted exchange. The
endpoint targets remain initialized. Descendant witnesses destroyed by
either endpoint's write die. A mere cone equality proves no safe exchange
relation. OP-12 additionally keeps its no-prefix-writing-callee condition,
and SET-1 keeps its post-right-hand-side target revalidation.

[WP-9] PAR-1 and PAR-2 use the same settled target descriptions and overlap
judgment as WP-3/EFF-5. A widened access has its full anchor cone as the
default interference footprint; it is not a read or write of just the
anchor's immediate fields. Same-target finite-suffix separation is available
only when the target identity is stable in the common comparison state.
Independent cursors under the same anchor have no such relation.

Include target evaluation, link loads, index and descriptor reads, argument
evaluation, by-value consumes, and ordinary binding/dataflow dependencies.
A reference rebinding writes no WF storage but defines a new reference
binding value; an overlapping execution may not read the old binding when
sequential execution uses the new one. Root reads are allowed sequentially
and still conflict with a possibly overlapping write for PAR. Keeping a
reference valid across a primitive store does not make that store race-free.

A structural mutation's footprint covers the owned storage it may destroy
through its written path. A standalone derived release keeps STOR-8's
absence of a state-row entry; ownership, validity, and the requirement to
retain storage for the complete overlapping execution still apply. No
allocator effect or synthetic lock is introduced.

PAR-2's permitted source forms, accumulator condition, control restrictions,
and affine-element/range-partition proofs are unchanged. An unknown-depth
descent is not itself an affine element map. Two ordinary iterations using
one loop-carried cursor gain no permission. A separately verified helper on
each already permitted disjoint range can retain that range as its anchor,
subject to every existing PAR-2 premise. Failure of permission leaves an
accepted program sequential; it does not reject it or insert a runtime edge.

## Termination and the safety invariant

The growing-definition graph has finitely many source nodes and edges.
Collapsing its SCCs leaves an acyclic graph. Selector chains between summary
boundaries consist of written suffixes on that graph, so their lengths are
bounded by the finite source, not by the runtime number of links. A prefix
meet can be computed on the graph without enumerating all paths through
branches. For each summary/root pair, an absent root can be added once;
after it is present, its anchor can only lose a finite number of selectors.
Summarized tails never grow again. Index disagreement drops a selector
rather than enumerating its values. Thus the cover equations have finite
height, and the specified monotone worklist terminates. Per root there is
one containing prefix, not a disjunction of arbitrarily many heap shapes.

Validity, writability, and the retained relations between the finite
reference definitions are finite bits/relations with must joins. The
relative suffixes retained between summary boundaries come from that same
acyclic source graph; a cyclic relative path is summarized rather than
unrolled. Numeric proof checking uses the existing terminating families and
does not form a second numeric widening loop. A one-more-pass claim does
not follow from finite height. This argument establishes termination of the
new path analysis, not a benchmark of its implementation or a new complexity
theorem about every existing compiler judgment.

The invariant for a valid widened `p: &T` is:

> In every represented concrete state, p designates one initialized existing
> T place at a finite owned descendant path of a recorded live anchor. Its
> hidden selections still identify that same place. Its root has not moved
> or ended scope. Every permitted write respects the recorded writability.
> The checker asserts equality only for established target identities, and
> asserts separation only for paths whose concrete denotations are disjoint.

The invariant quantifies over every alternative, not just the actual path
on one tested execution. It assumes the base language preserves an acyclic
single-owner forest, that no stored references or hidden aliases bypass it,
and that linked definitions obey their declared semantics. WP-8 is needed
to preserve that forest during exchange.

Formation establishes the invariant using the old reference's witness and
the checks on the new suffix. Joins and cones enlarge possible locations
without claiming new equality or validity. Reads preserve the forest.
Primitive stores cannot remove the witness path. A content write at or
below the captured target cannot destroy a proper ancestor in a forest;
other potentially destructive writes revoke the witness before its next
use. Calls apply that argument to their declared paths, and scope edges
apply it to their releases. Support kills and distinct target identities
prevent a correct pointer from carrying a false field or domain proof.
Conservative footprints prevent a sequentially safe invalidation from
being reordered into a race. These are an informal preservation argument,
not a mechanized proof of the amended kernel.
