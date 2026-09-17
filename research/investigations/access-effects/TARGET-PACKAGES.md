# Target packages: proposed x0 extension E1

This note proposes a small research extension to the frozen x0 rule sheet in
[MATRIX-X0.md](MATRIX-X0.md). It is not an x0 amendment, an official language rule, or a
claim that general existentials, containers, or resource predicates are
closed. E1 only lets finite fixed-object programs hide, copy, open, return, and
repack one captured locator target. R1-R16 otherwise remain unchanged.

## Alternatives and criterion fixed before selection

The extension must recover a source snapshot's target correlation across copy
and field rebinding; reject stale `Live`/`Full` facts and duplicate disposal
use; expose exact whole-call targets without treating existential names as
disjoint; and erase every proof-only component. It should discharge concrete
1-8, 8-20, 8-21, and 8-22 cases without inventing dynamic resource families.

1. Package `exists P. Loc<P,T> & Live(P) & Full(P)`. Copy then appears to copy
   mutable validity. Release through another alias would require invalidating
   facts inside every copy: a larger dependency system with resurrection risk.
2. Give every open a fresh abstract name. This loses equality between two opens
   of one snapshot. Considering the names distinct is unsound; leaving them
   unrelated loses same-target state correlation and precise effect comparison
   (although unknown read/read remains compatible).
3. Package an immutable target image and keep mutable state in the current
   context. Copy preserves the image; open reveals it under a scoped name.
   `Live`, layout, `Full`, duties, and separation remain operation-time facts.

E1 chooses alternative 3. It adds a value-to-target relation, not a general
resource logic.

## Forms and static meaning

```text
TargetPack<T>                 // exists P. Loc<P,T>
OwnedTargetPack<T>            // exists P. Owner<P,T>, including P's duty
pack_target(p)                // p: Loc<P,T>
pack_owner(move(a))           // a: Owner<P,T>
open_target e as <P,p> { s }
open_owner move(e) as <P,a> { s }
repack_target<P>(p)
repack_owner<P>(move(a))
```

The spelling is provisional. `TargetPack<T>` is Copy when its executable
locator/descriptor fields are Copy. `OwnedTargetPack<T>` has its owner's class;
hiding never makes it Copy.

The checker records `image(k) = target(p)`, where k identifies an immutable
value snapshot in the checked program. It is not a runtime package ID, address,
generation, or occupancy flag. Copy preserves k. Loading from mutable storage
creates a snapshot tied to that load result; rebinding storage does not alter
the snapshot.

`TargetPack<T>` contains no `Live(P)`, current compatible layout, `Full(P)`,
`Empty(P)`, disposal duty, or separation fact. R3-R8 check those in the current
context at each operation. Immutable type information needed to interpret
`Loc<_,T>` may cross the boundary; current layout compatibility may not.

An opaque caller need not know a finite origin set: a `TargetPack<T>` parameter
still has exact symbolic `image(b@entry)`. It is neither fresh nor a wildcard,
and two unrelated parameter images may alias.

## Introduction, opening, and repacking

From `p: Loc<P,T>`, `pack_target(p)` produces `TargetPack<T>` with the same
executable representation and an erased image equation. It neither reads P nor
proves P live, full, separate, or disposable. A naked locator never captures a
duty held elsewhere.

From `a: Owner<P,T>` carrying P's duty, `pack_owner(move(a))` moves owner and
duty into `OwnedTargetPack<T>` and empties the source under R5. Packages may
preserve explicitly named immutable structural relations, such as P being a
fixed field. E1 excludes arbitrary predicates, mutable state, quantified
families, and inferred reachability.

`open_target e as <P,p> { s }` evaluates e once to snapshot k, introduces P for
`image(k)`, and binds p to its copied locator. It adds no state or duty fact.
Opening one snapshot twice, including copies, reveals equal targets:

```text
let b = pack_target(loc(x)); let c = b
open_target b as <P,p> { open_target c as <Q,q> { /* P = Q */ } }
```

That equality does not make P fresh or separate. Independently introduced or
loaded packages may alias; different binders prove nothing. Repeated opens of
two aliased opaque holders likewise give no separation unless their evaluated
holder images or checked relations establish equality or separation.

An open name cannot escape naked. A retained locator, closure capture, returned
value, or latent effect must be repacked or quantified by a checked enclosing
type/contract that binds P. `open_owner move(e)` consumes its package and
reveals the one owner and duty. Every exit must release, transfer, or repack it.
For an affine owner, an existing R7 cleanup may discharge it when one verified
unconditional sequence fits the exit; a linear duty still needs explicit
discharge or transfer. E1 has no nondestructive owner open; that needs a larger
borrow subsystem.

`repack_target<P>(p)` requires p's captured target to be P and all declared
immutable schema relations. It needs neither liveness nor fullness, so a dead
locator may be repacked but remains unusable. `repack_owner<P>(move(a))` also
requires a valid owner carrying P's available duty. A naked locator, copied
owner, or consumed duty cannot satisfy it.

Writing a package/holder field changes only that field's current image. Earlier
snapshots retain theirs. R8 invalidates facts about the mutable field through
possible aliases; target facts change only when a resolved operation may affect
that target. Copy duplicates locator bits and correlation, never state,
contents, exclusion, separation, or duty.

Argument evaluation remains source ordered. If evaluating a later argument
rebinds the holder, ends P, empties P, or consumes its duty, the invalidation is
applied before the call or repack is checked. A saved locator still targets old
P, but no stale state or duty premise survives.

## Calls, results, state, and effects

A checked call may return a hidden target and establish current facts. This
small body derives, rather than merely declares, the result relation:

```text
fn hide_live<P>(p: Loc<P,Int>) -> TargetPack<Int>
  requires Live(P), Layout(P,Int), Full(P)
  ensures image(result)=P, Live(P), Layout(P,Int), Full(P)
  accesses none
{ return pack_target(p) }
```

Opening the result binds P and makes its verified poststate available in the
caller's current context. Facts are not stored in the Copy package. Release of
P through any alias, including a locator copied from a nested open, consumes
the one duty, ends P, and invalidates all current state facts. Reopening an old
copy yields the same P without access premises. Content state and allocation
duty remain separate: `take(P)` can make P empty while leaving its duty; moving
the duty does not make content full.

The callee verifies postconditions for the selected result. `pack_target` alone
establishes only the target relation. A branch-selected result denotes the
selected target and must establish facts in every alternative. R9 may retain a
finite correlation; E1 adds no general disjunction solver. A naked package
cannot extend an inline local's lifetime.

Effects cross the existential boundary as a dependent row, for example:

```text
exists P where P=image(b@entry). reads(P)
```

The same rule verifies a genuinely opaque input without enumerating origins:

```text
fn observe_hidden(b: TargetPack<Int>) -> Int
  requires Live(image(b@entry)), Layout(image(b@entry),Int),
           Full(image(b@entry))
  accesses reads(image(b@entry))
{ open_target b as <P,p> { return read(p) } }

fn mutate_hidden(b: TargetPack<Int>, v: Int)
  requires Live(image(b@entry)), Layout(image(b@entry),Int)
  accesses writes(image(b@entry))
{ open_target b as <P,p> { write(p,v) } }

fn hide<P>(p: Loc<P,Int>) -> TargetPack<Int>
  ensures image(result)=P
{ return pack_target(p) }
```

Opening substitutes scoped `P = image(b@entry)` into the body. Closing the open
projects internal `reads(P)` back to the external dependent row above; P never
escapes free. Thus separate alpha-renamed opens remain comparable at calls, and
the caller discharges state requirements without knowing a concrete origin. A
by-value parameter's local copy adds no caller-storage effect. Evaluating an
actual such as `observe_hidden(h.at)` separately records the caller's read of
`h.at` before entering the call.

Within an open, an effect on P projects through the package/holder image to that
exact hidden target. It is not a wildcard. Whole-call summaries include
argument evaluation, descriptor reads, projected target accesses, transfers,
and end effects under R12. Effect paths resolve against evaluated entry/result
images; later rebinding does not retarget them. Restoring contents does not
erase an intermediate write, hole, transfer, or end.

Two reads through one snapshot or its copies are overlap-compatible.
Read/write, write/write, and end/access on that image conflict. Unrelated
packages have unknown alias relation, so a possible conflict denies requested
overlap but does not reject valid sequential calls. May-alias comparison does
not replace the exact dependent effect row. Checked equality or separation may
decide it; distinct existential names may not.

If a contract omits the relation needed for exact projection, checking rejects:
`hidden target relation unavailable for effect projection`. E1 adds no unknown
memory wildcard.

## Erasure, cost, and named losses

`TargetPack<T>` lowers to ordinary locator/descriptor fields;
`OwnedTargetPack<T>` lowers to the existing owner descriptor. Opens, repacks,
names, image atoms, equalities, state facts, and duties erase. E1 adds no runtime
generation, occupancy, ownership, alias, or liveness flag; registry, map,
allocation, lock, branch, or scheduling edge; or pointer-wide `noalias`.
Runtime cost is only the source's locator/owner operations. Checker cost is
finite provenance equality plus existing fixed-name R8/R9/R11-R14 reasoning.

The checker never reconstructs lost provenance from addresses or bodies. It
rejects by naming the unavailable package-image correlation, current
Live/layout/Full fact, disposal duty, or target equality/separation.

## Minimal challenge witnesses

These are research pseudocode, not runnable tests.

1. `b=pack_target(loc(*a)); c=b`: accept copy; reject release through c without
   the separately available duty. Cost: pointer copies.
2. `ob=pack_owner(move(a)); reject copy(ob); open_owner move(ob){release}`:
   accept one release. Cost: existing owner move and release.
3. `b=pack(loc(x)); c=b; open b as P; open c as Q`: derive P=Q; permit read/read
   overlap when current access premises hold.
4. `b=pack(loc(*a)); c=b; release(a); open c { reject read }`: copied image does
   not copy Live; reject for missing current state.
5. `h.at=pack(loc(x)); old=h.at; h.at=pack(loc(y)); open old { write(7) }`:
   with current premises write x, not y.
6. `open b as <P,p> { return repack_target<P>(p) }`: accept; a later open
   correlates with P but needs current state.
7. `b=hide_live(loc(*a)); open b as <P,p>{read(p)}` accepts from
   `hide_live`'s derived operation-time poststate.
8. `b=hide(loc(*a)); q=loc(*a); release(q); open b { reject read }`: hide's
   result relation proves aliasing, so nested/copied release invalidates facts.
9. `open u as P; open v as Q; overlap(read(P),write(Q,1))`: deny overlap for
   unknown alias relation; keep valid sequential execution.
10. `b=if c {pack(loc(x))} else {pack(loc(y))}; open b {branch c; read}`:
    accept with R9 correlation. If it is lost, accept only if the remaining
    facts independently prove every possible target live/layout/full; otherwise
    reject and name the missing operation-time fact.
11. `c=b; overlap(observe_hidden(b),observe_hidden(c))`: permit when complete
    dependent effects are compatible reads.
12. `overlap(observe_hidden(b),mutate_hidden(b))`: deny exact read/write
    conflict even if mutation restores the entry value.
13. A hidden-target function declared `accesses reads(?)` rejects with
    `hidden target relation unavailable for effect projection`, not widening.
14. `slot x=1; return pack_target(loc(x))` may return an inert package. A
    declared `ensures Live(image(result))` rejects because x ends at scope exit;
    a caller read without that poststate also rejects.

Additional adversarial checks are mandatory: take P then confirm its duty
remains; evaluate an opaque `TargetPack` input with no finite origin identity;
and reject a closure or effect clause that mentions an open P without an
enclosing binder.
Source-order invalidation has this concrete form:

```text
fn consume(x: TargetPack<Int>, ignored: Unit) -> Int
  requires Live(image(x@entry)), Layout(image(x@entry),Int),
           Full(image(x@entry))
  accesses reads(image(x@entry))
{ return observe_hidden(x) }

// q targets image(b); its available duty is transferred into this block;
// image(b) is Live, layout-compatible, Full, and has disposable contents.
consume(b, { release(q); () })
// later argument ends image(b); consume's entry requirement fails
```

Inside an open, `kill(q,duty)` with `target(q)=P` invalidates Live: later read and a
Live-bearing return reject, while target-only repack remains safely inert.

## Matrix impact and reruns

E1 directly affects frozen cells **1-8; 8-8 through 8-24**. Frozen verdicts do
not change in place. The rerun scope included the hidden generalization of 1-8
and hidden forms of **8-8, 8-9, 8-10, 8-12, 8-13, 8-16, 8-18, 8-20, 8-21,
8-22, and 8-24**. Their existing monomorphic D witnesses are non-regressions:
E1 must not add locator duties, retarget old loads, revive ended storage, lose
branch capture, omit field reads, or emit uniqueness metadata.

E1 may derive finite hidden forms of **1-8, 8-20, 8-21, and 8-22** when their
premises satisfy these rules. **8-14** remains U for provider formation and
management effects. **8-15, 8-17, and 8-23** remain U for general predicates,
loop-carried dynamic families, and cross-iteration family separation. **8-19**
remains U for general higher-order contract refinement, though a direct finite
callback contract can use E1.

Also rerun unchanged boundaries **1-7, 1-9, 1-10, 3-8, 4-8, 7-8, 9-10,
9-21, 10-22, 16-21, 18-22, 21-22, and 22-24**. They guard locator/duty
separation, global end invalidation, whole-call effects, and conservative
lowering.

The local hand-rerun against E1 produced these actual results:

| E1 witness | Frozen cell(s) | Result under E1 | Deciding ground |
|---|---|---|---|
| 5, 6 | 1-8, 8-20, 8-21 | D | snapshot introduction/open/repack and exact invalidation |
| 11, 12 | 8-22 | D | dependent effect-row closure; read/read allowed, read/write denied |
| 1, 3 | 1-7, 3-8, 4-8, 7-8, 8-8 | D | Copy preserves image but carries no duty or state |
| 4, 8 | 1-9, 8-9, 8-10, 9-10 | D | end through any resolved alias invalidates Live globally |
| 5 | 8-12, 8-13, 8-21 | D | stored-value rebind leaves the loaded image unchanged |
| 10 | 8-16 | D | finite guarded result correlation under R9 |
| opaque contract above | 8-18, 8-20, 8-21, 8-22 | D | substitute image at open; project P on close |
| target package erasure | 8-24 | D | lower the executable locator only; image and state facts erase, with no `noalias` |
| provider witness | 8-14 | U | provider formation and management-state model still missing |
| dynamic container/loop | 8-15, 8-17, 8-23 | U | predicate/family and cross-iteration rules still missing |
| general callback | 8-19 | U | general higher-order contract refinement still missing |

These are local explanatory derivations, not changes to the frozen matrix or a
completeness proof. No claim is made for cells not listed in this table.

## Bounded challenge outcome

A Sol challenge of this E1 scope required four corrections: a locator package
itself carries no validity guarantee, and one targeting an ended local becomes
unusable even though it may escape as inert target information;
opaque input contracts bind and externally project `image(b@entry)`; only
`OwnedTargetPack` carries a duty while mutable state stays outside both package
forms; and caller argument evaluation contributes effects and invalidation
before callee entry. The revised witnesses above include those boundaries. The
challenge found no concrete unsafe accepted case within the finite
single-target subsystem. That is a bounded explanatory result, not a theorem.

Unwitnessed combinations outside the rerun table remain pending rather than
derived. In particular, E1 supplies no result for arbitrary nesting, dynamic
families, or general higher-order refinement.

E1 does not define arbitrary heterogeneous containers, inductive predicates,
quantified target/resource families, family membership or separation,
conditional aggregate cleanup, general existential subtyping, or independence
across dynamic iterations. Those remain explicit U results. Concurrency and
the current capability floor remain full-candidate obligations; E1 supplies
only exact single-target projection and conservative alias comparison.
