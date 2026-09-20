# Wildcard paths: findings and recommendation

Do not implement the three proposed sentences as a complete rule. The
subtree-summary idea is plausible and useful for iterative owned-link
traversal, but the stated proposal omits necessary semantics. The most
immediate failure is not an exotic race: under a literal additive reading
of v0.60, leaving the Some arm invalidates the newly descended reference.
The proposed loop is still rejected. Making it work requires an explicit
change to the treatment of already traversed refinement and bounds evidence.

Recommend the conservative amendment in [RULES.md](RULES.md): a typed
subtree cover, a separate current-target identity, a checked hidden-path
existence witness, and a deterministic finite summary analysis. Keep the
existing no-escape, no-hole, readonly, ownership, and proof-domain rules.
Do not add a runtime check or a general heap-shape prover. Its main cost is
loss of precision for independently selected cursors in one subtree.

The decisive findings are:

1. **Widening does not by itself preserve a path through a match.** REF-2
   and ENT-3.S15 lose the payload-selection fact on arm exit. C01 and C15
   distinguish a safe address from the source refinement that selected it.
   The amendment seals only the existence of the already selected place;
   selecting another payload still needs a current match.
2. **A summary is a may-alias set, not an equality of places or values.**
   Independent p and q can both have cover R.** while their tags are 1 and
   0. Replacing ENT-2's distinct source-term identities with one summary
   term admits division by zero (C35). The current rule already prevents it.
   Rebinding within one cone must also kill facts about the old target
   (C33). Primitive stores preserve a pointer, not its field facts (C34).
3. **Ancestor events and operation-specific storage changes still count.**
   Root moves, containing Box replacement, owner scope exit, and window
   removals cannot be handled by testing only writes below R. C16-C19,
   C22-C23, and C36 provide concrete disappearing-place witnesses.
4. **Calls need origin-relative preservation, not argument immunity.** A
   reference passed to a callee is not protected from another actual's
   ancestor write. C12 shows this even without widening. Conversely,
   erasing all row suffixes to R.** rejects a safe helper with disjoint
   sibling-field effects (C09). Retain the current target and finite suffix.
5. **Swap's allowance is equal-or-disjoint, never ancestor-or-descendant.**
   C21 can create a self-owning Box cycle if the equality exception is
   implemented as permission for every overlapping same-type pair.
6. **"Recheck once" is not a termination or convergence argument.** The
   p/q/r propagation in C29 needs further propagation to discover an
   aliased write pair. Mutual reference-flow cycles and nested loops must
   converge by a finite domain, not by a pass count (C27-C29).

These findings have different evidential strength. C01 is a direct conflict
between the desired behavior and the current normative rules. C29 refutes
the literal one-recheck promise in the todo entry. C12, C14, C16, C21,
C33-C36 refute the explicitly identified permissive interpretations.
I have not found a mandatory new use-after-free accepted by all coherent
readings of the three rules *when every inherited v0.60 check is also
enforced*. It would be misleading to label an ancestor move a counterexample
without acknowledging REF-2, or to label an unguarded Some selection
accepted. The proposal is incomplete, and several natural completions are
unsound; that is more precise than claiming every completion is unsound.

## Scope, basis, and selection ground

The examined base is commit `de78fb19c2ad7bbe54dc2146a3b350e10f612067`,
branch `research/x1-wildcard-path`, with
[kernel v0.60](../../../spec/kernel-spec.md) active. The proposal is the
[wildcard-path todo entry](../../../docs/todo.md), read against REF-1..4,
OWN-7, EFF-1..5, PAR-1/2, and the relevant operation, fact, and contract
rules. The existing [ownership decisions](../../../design/language/ownership/)
and [ownership parent](../../../design/language/ownership.md) explain the
current static-shape refusal, single-owner values, absence of stored
references, and effect-based call boundary. They supply historical design
grounds, not replacement language definitions.

The evaluation criterion is deductive: admit ordinary single-cursor descent
and a no-hole edit at the selected link slot, preserve every existing safety
condition, reject concrete alias/destruction countermodels, and give a
terminating deterministic checking procedure independent of resource
budgets. No timing experiment selects this recommendation. No benchmark or
implementation-cost claim has been measured. The todo entry's claim that
this costs the compiler almost nothing is therefore not established here.

This criterion fits the [constitution's](../../../docs/constitution.md)
required static safety and runtime-performance objectives. It does not
make this particular abstraction uniquely necessary. Recursion, explicit
indices, a frozen-subtree discipline, and richer shape proofs remain real
alternatives, compared below.

The three files serve the same investigation: REPORT explains the choice,
RULES makes it reviewable, and CASES supplies the countermodels and expected
verdicts. They belong in the existing investigations home. Supersede or
remove them if a later investigation replaces their proposed mechanism;
retain them while these arguments remain useful design evidence. They are
not a daily test dependency, an implementation inventory, or a conformance
change. No active specification, design-tree node, compiler, or test has
been changed, and no commit or push is part of this work.

## Why the small abstraction needs several distinctions

### A cover is not the selected object

At runtime a cursor is one ordinary reference, not an iterator over every
place in its cone. At compile time its concrete path can grow without a
finite exact path set. The cone forgets that path while preserving an
overapproximation of where the selected place can be. It includes R itself
to represent the initial iteration and a reset.

That loss of information is safe for a negative permission judgment:
"these may overlap, so do not run these writes concurrently." It is unsafe
for a positive identity judgment: "these have the same summary, so a fact
about one proves a fact about the other." It is also insufficient for a
positive disjointness judgment between two arbitrary descendants.

ENT-2 already distinguishes source-term identity from storage overlap, and
the amendment retains that distinction and its term-eligibility boundary.
It also keeps a small amount of ordinary local target identity:
the same unrebound p is the same target; an alias snapshots it; a projection
has a finite suffix relative to it. Different fields of that one target
remain separate. Independently found targets remain possibly overlapping.
This is not pointer equality at runtime or an analysis that discovers a
path between two arbitrary nodes.

### A witness survives only mutations that cannot remove its path

For a valid `p: &T`, some finite ownership path under its anchor reaches
the same initialized T place the reference captured. An ordinary primitive
field store cannot remove that path. A content replacement of p cannot
destroy its proper ancestors in an acyclic ownership forest. A mutation
through another unknown cursor may destroy an ancestor and must invalidate
p before its next use. These are the actual grounds of the three rules.

The literal rule "invalidate every other reference" is safe but loses
even a proved alias of p across a content write. RULES preserves a
definitely equal or ancestor target using the same finite suffix relation;
it still kills unrelated cursors with overlapping cones. This extra
precision has a local syntactic witness, not an inferred heap invariant.

The hidden prefix contains more than field names: a Box ownership edge, a
selected variant, an initialized array/window position, possibly a range,
and readonly restrictions can all matter. The witness must retain their
validity consequences even when it forgets their spelling. In particular,
dropping a lexical Some fact is harmless to an unchanged selected address;
changing the owning enum to None is not. A new Some selection is never
justified by an old hidden witness.

### Validity, proof support, and parallel safety answer different questions

The primitive exception cannot apply to proof support. If root.tag becomes
zero, a summary cursor that might name root loses a previously proved
nonzero tag. The pointer can remain valid while a division through it is
no longer justified. Writability must also survive abstraction: reaching
a Node through a readonly field does not become a mutable Node reference.

Likewise, allowing a root read while a cursor lives says nothing about
concurrent execution with a possibly overlapping cursor write. PAR must
retain the complete cone footprint by default and the ordinary dataflow of
cursor rebinding. It may exploit separated anchors or stable same-target
field separation; it may not infer iteration independence from "descent".
Existing PAR-2 source-form restrictions still apply even to a program whose
nodes are mathematically distinct on successive iterations.

### Calls and swaps must not erase the distinction

A callee row is still its complete runtime-effect boundary. A caller
substitutes the actual's selected target and suffix, checks effect pairs,
then invalidates references and facts, then publishes only eligible
verified contract relations. Passing both root and cursor does not imply
that both have effects: C12's unused cursor creates no pair to compare.
Its presence must therefore not exempt it from a destructive root write.

For swap, possible equality between two array slots is benign. Possible
proper ancestry is not. A broad overlap relation deliberately cannot
distinguish those without additional established target structure. The
safe exchange test needs equal-or-disjoint storage, with no possible
ancestor relation, independently of the later invalidation test. This
clarifies an existing boundary as well as its wildcard extension.

## Prior art and what it teaches WF

These comparisons cite primary papers or project documentation. The lessons
for WF are inferences, not claims that those systems implement this proposal.
Historical examples are identified as such; no claim about a current
compiler's acceptance is inferred from an old example.

| Work | Relevant mechanism and source | Lesson for WF |
| --- | --- | --- |
| Rust NLL | The rustc guide describes MIR-based borrow checking and control-flow-based region inference. [Borrow checker guide](https://rustc-dev-guide.rust-lang.org/borrow-check.html). | The place that a reference can reach and the control points where it can be used are different dimensions. WF need not import Rust lifetimes, but cannot replace validity flow with a subtree bit alone. |
| Rust two-phase borrows | Certain implicit mutable borrows have a reservation stage and an activation stage; shared access can occur between them subject to conflict checks. [Two-phase borrows](https://rustc-dev-guide.rust-lang.org/borrow-check/two-phase-borrows.html). | This explains argument-evaluation flexibility, not recursive path widening. WF already evaluates and revalidates assignment targets under SET-1. C36 needs that discipline; calling wildcard descent "two-phase borrowing" would obscure the missing path invariant. |
| Polonius cursor problem | The 2023 Rust team update gives issue #47680's `temp.maybe_next()` loop, where a loan returned on one branch flows into the next iteration. It contrasts loan-set modeling and location-sensitive flow with the earlier analysis. [Polonius update, 2023](https://blog.rust-lang.org/inside-rust/2023/10/06/polonius-update/). | WF avoids the returned-reference part through REF-3, but has a related flow problem when a local name becomes a descendant. Rebinding, joins, and arbitrary subsequent iterations must be modeled together. The Rust example is not evidence that a WF cone handles conditional loan returns. |
| Cyclone regions | Cyclone combines a region-based type/effect discipline with region lifetimes, including stack allocation. [Grossman et al., PLDI 2002](https://doi.org/10.1145/543552.512563); the project's guide explains region allocation/deallocation. [Regions guide](https://cyclone.thelanguage.org/wiki/Introduction%20to%20Regions/). | Keeping an allocation region alive is not a proof that a particular enum payload or window element still exists. R.** is a containment abstraction over ordinary owned values, not a new allocation region. Importing a region lifetime would not remove the structural invalidation obligation. |
| Mezzo | Duplicable and affine permissions describe aliasing and mutable ownership. The paper's adoption/abandon mechanism uses dynamic ownership checks, whereas its static discipline separately controls permissions. [Pottier and Protzenko, Programming with Permissions in Mezzo](https://gallium.inria.fr/~fpottier/publis/pottier-protzenko-mezzo.pdf). | Distinguish knowing an object from owning permission to access its current representation. Retaining one target witness while invalidating other uncertain witnesses has a static permission interpretation. Dynamic adoption tests cannot be imported as WF acceptance fallbacks, since they can fail at runtime. |
| Vale region borrowing | Vale's region guide describes read-only regions and explicitly labels these region features planned. Its memory-safety page distinguishes generational-reference checks and planned region-borrow mechanisms. [Regions](https://vale.dev/guide/regions), [Memory safety](https://vale.dev/memory-safe). | Freezing a whole region makes cheap references plausible but disallows the structural edits WF wants through one cursor. Runtime generational validation is a different tradeoff from WF's proof-only acceptance. The cited pages support a design comparison, not evidence of a completed zero-overhead implementation. |
| Austral | The specification gives explicit region-scoped borrowing; the borrowed owner is unavailable inside the borrow block, and the alias cannot leave it. [Austral specification, Borrow Statement](https://austral-lang.org/spec/spec.html#borrow-statement). | WF's free root reads and destructive invalidation deliberately choose a different policy. Austral demonstrates a simpler scoping boundary; copying that boundary would remove some of the requested expressiveness rather than explain how to preserve it. |
| ATS views | ATS uses linear propositions such as `T@L` to witness memory at a location and dataviews to describe recursive resources. [Hongwei Xi, Introduction to Programming in ATS, chapters 13-15](https://ats-lang.sourceforge.net/DOCUMENT/INT2PROGINATS/HTML/INT2PROGINATS-BOOK-onechunk.html). | A checked existence witness and an ordinary value are different things, and proof erasure need not mean proof absence. Richer multi-cursor edits could require explicit resource proofs. WF's current affine `invariant` and finite `use` vocabulary does not already provide arbitrary ATS-style heap views. |
| Separation-logic list segments | Reynolds explains reasoning about disjoint mutable heaps. [Separation Logic, LICS 2002](https://www.cs.cmu.edu/~jcr/seplogic.pdf). The Verifiable C tutorial describes a partial list using a segment or a quantified magic-wand frame. [Verif_append2](https://www.cs.princeton.edu/courses/archive/spring23/cos510/sf/vc/Verif_append2.html). | A prefix segment plus a focused node/suffix records what R.** discards: where the traversed part ends and which storage is separate. This is a plausible route to two-cursor splicing and structural contracts, but it requires a new checked heap-predicate fragment, not merely a longer list of invalidating writes. |
| Shape analysis with summary nodes | Sagiv, Reps, and Wilhelm describe summary nodes for indistinguishable concrete cells, conservative three-valued information, and materialization for a selected cell. [Parametric Shape Analysis via 3-Valued Logic, TOPLAS 2002](https://lara.epfl.ch/w/_media/projects/sagivetal02parametric.pdf). | R.** is a very coarse summary of possible descendants. Its selected cursor still needs separate identity. Treating the whole summary as a singleton or making a strong fact update for every represented cell is unsound. WF can use a much smaller domain because its owned heap is a forest, but must still define concretization, joins, transfer, and finite convergence. |

The last comparison is particularly close. The recommendation does not
adopt TVLA's full predicate vocabulary, materialization search, or shape
graph domain. It keeps one live selected target plus a conservative cover
and bounded local relations. The need to distinguish them follows from
C09 and C35, independently of whether any particular prior-art algorithm
is used.

## Alternatives and remaining expressiveness limits

The following limits are properties of the proposed design or of unchanged
WF rules, not compiler bugs. The table distinguishes what the program would
need from what this investigation recommends adding now.

| Program or requirement | Why a wildcard alone does not express it | What would be needed |
| --- | --- | --- |
| Two independently found cursors in one list, one performing a structural edit while the other remains usable | One may be a destroyed descendant of the other; R.** contains no separation relation. | Rediscover the second cursor after the edit, use independently separated subtree anchors, or add verified list-segment/reachability separation. |
| Linked-list splice or tree rotation that simultaneously mutates ancestor and descendant through separate actuals | The call boundary needs a proved interference relation; invalidating afterwards is too late. | One cursor plus an owned, no-hole local transformation, or a checked focused-context/resource proof. A rotation implemented by an OP-12 owned transformation remains possible. |
| A cursor that follows its object across root move, replacement, or reparenting | REF-2 names an ownership path, not a stable object identity; the old path dies. | Reform from the new owner, use handles with application-level identity, or explicitly redesign reference identity and its proof obligations. |
| A saved cursor surviving vector growth or Ring reindexing | Its captured address or logical selection can cease to name the same place. | Recompute from an index after mutation, or a storage/handle discipline with a separately proved stable-address guarantee. |
| Arbitrary DFS with a stored stack of references, a returned cursor, or a stored iterator | TYPE-8/REF-3 prohibit reference storage and escape. | Recursion, an owned stack of path choices/indices, a zipper holding owned context, or a separate change to the no-stored/no-returned-reference design. |
| Doubly linked raw references, parent pointers, shared cyclic graphs | Owned values cannot store references and the safety argument uses a forest. | An index/handle representation, or a new graph ownership model with an independent soundness argument. |
| Proof that a search found a particular node, that a list is sorted, or that a traversal visits every node once | A cone is a may-location cover, not an inductive functional invariant. | Suitable checked inductive predicates and contracts, or an index representation within the existing numeric proof fragment. |
| A generic whole-Node effect row that preserves all unknown descendants because this implementation only changes tag | Callers see the declared row and may not inspect the body. | A narrower declared field row where applicable; a separate structural-preservation contract vocabulary for more general cases. |
| Parallel iterations of one cursor walk | Unknown-depth descent is not an existing PAR-2 partition proof and p is loop-carried state. | Existing disjoint array/range partitions, explicit owned subdivision, or a new finite proof family for disjoint heap segments. |
| Arbitrary independent cursor alias/equality tests | Cone equality supplies no target equality and addresses are unobservable. | Use application data or handles; do not add a pointer test solely to repair an unsound static inference. |
| Guaranteed bounded stack/time for recursion or a terminating traversal | WF currently has no general termination proof and a wildcard says nothing about progress. | A separately specified termination/resource proof or the proposed independent tail-call mechanism. |
| Mutation through a readonly hidden prefix, replacing a linear value by dropping it, or leaving a hole between statements | These violate unchanged TYPE-2, PROV-6, and WIN-3 constraints. | Change the program's ownership/operation structure. Widening is not a reason to relax those invariants. |

Pure recursion is the smallest existing alternative: it needs no new path
domain and makes each step's reference local to one call. It does not give
an iterative stack bound without a tail-call guarantee, and the current
language's proposed `musttail` work is separate. A pool plus indices offers
fixed-shape paths and useful numeric disjointness, but changes the data
representation and does not demonstrate that every owned tree should be
rewritten that way.

A frozen-subtree variant would forbid structural writes while any cursor
is retained, while allowing reads and selected primitive writes. Its
preservation argument is simpler, but it excludes the requested link-slot
removal. It is a reasonable smaller research baseline, not an equivalent
implementation of this proposal. A full separation-logic or shape-analysis
solution can retain more information but is not justified for single-cursor
descent alone. The recommended intermediate design should be reopened if
real programs repeatedly need the information the cone discards.

## Proposed design revisions and affected consumers

These are pending proposals recorded here under the user's write boundary,
not edits or owner rulings on the live tree. No approval is requested to
finish this research. The existing tree remains authoritative for its
current decisions until a separately reviewed language amendment is chosen.

Node: design/language/ownership/reference-rebinding.md

Decision: Replace the static-shape prohibition for growing local reference
flow with a finite typed cone and current-target identity, using the WP-6
component construction, because C01/C02/C37 need an iterative selected
place while C27-C29 show why exact path enumeration and one body recheck
are insufficient, instead of requiring recursion or a pool index for every
owned-link walk.

Rejected: A fixed number of loop rechecks cannot be the acceptance rule,
because C29 can delay a new root alternative through arbitrarily many
reference bindings. A general heap-shape solver is not selected because
single-cursor existence needs no inferred relationships between arbitrary
nodes.

Node: design/language/ownership/reference-validity.md

Decision: For widened paths, package checked hidden selections in an
existence witness and preserve it only under WP-4, because a match arm's
exit does not destroy its selected child but a replacement of that child's
owning prefix does, instead of requiring the old source refinement to
remain lexically available forever or letting every write through a
reference preserve all of its descendant evidence.

Rejected: Extending this witness to ordinary field values or newly written
payload selections is unsound by C14 and C33-C35. Preserving arbitrary
other cursors on a structural write needs separation evidence the cone
does not retain. Known equal-target aliases can be preserved on the
ordinary content-write ground, as C41 shows.

Node: design/language/ownership.md

Decision: Extend the common overlap judgment with conservative cone covers
and a separate same-target finite-suffix judgment, because an unknown
descendant may alias a root or another cursor while two fields of one
captured target remain distinct, instead of treating a wildcard as an
ordinary equal-place token or making independent cursors disjoint from
their terminal field spellings.

Node: design/language/effects/call-site-check.md

Decision: Substitute a captured target and row suffix and apply invalidation
to every caller reference including actuals, with content preservation
relative to that target, because C09 needs ordinary field separation and
C12 shows that merely being an argument cannot protect a place from
another actual's ancestor write, instead of flattening every effect to the
same cone or granting all actual arguments blanket validity immunity.

Node: design/language/ownership/exchange.md

Decision: State the existing same-place exchange allowance as an
equal-or-disjoint requirement that excludes possible proper ancestry,
because swapping a Node with its owned descendant can form a self-owning
cycle as C21 shows, instead of reading overlap tolerance as permission
for every same-type pair. This is a clarification of the stated equality
ground, not a reason to withdraw same-array swaps with unknown equal
indices.

The affected specification set is REF-1/2/4, OWN-7, EFF-2/5, OP-11,
ENT-3.S15 and ENT-5's support treatment. ENT-2's source-term identity and
eligibility are retained. SET-1's revalidation, OP-10's
window consequences, OP-12's atomic condition, FN-8/9 and CALL-5/6's
transport, and PAR-1/2 consume those changes. Their existing constraints
are retained as described in RULES; they cannot be bypassed by the summary.
The relevant effects, proof, and parallelism decisions need corresponding
wording review if this amendment is adopted, not a silent implication
that today's compiler has these features.

No stored-reference or region decision needs reversal. The single-owner
model, scalar proof fragment, no-runtime-fallback policy, row exactness,
and current PAR-2 partition forms remain in force. The pending changes
are broader than "purely additive path syntax": they alter validity
evidence and its consumers even though they add no writer syntax.

## Validation and limits

All case verdicts are hand-derived against the cited rules. Positive
wildcard acceptance is a proposed result, not a compiler observation.
The argument in RULES establishes a finite path-domain construction and
an informal preservation invariant. It does not constitute a mechanized
soundness proof of the whole amended WF kernel.

No build, performance experiment, `make check`, commit, or push was run.
This investigation changes only the three Markdown files here. An in-memory
Perl scan passed (exit 0): exactly three ASCII files, 42 unique consecutive
case headings, all relative links and cited case IDs resolving, no trailing
whitespace, final newlines, and balanced code fences. `git status --short
--untracked-files=all` listed only these three new files (exit 0).
`git diff --check` produced no diagnostics (exit 0); the additional
`git diff --no-index --check /dev/null <file>` for each new file produced
no whitespace diagnostics (exit 1 denotes the new-file difference).
These checks cannot certify the semantic argument. Before implementing
the change, use C01/C09/C12/C14/C21/C29/C33-C36 as discriminating acceptance
and rejection requirements, alongside the already maintained ownership,
effect, and parallel conformance coverage.
