# Wildcard-path cases

These are specification arguments, not compiler test results. No WF program
below was compiled. The declarations and examples use v0.60 spelling; blocks
marked `text` are explicitly schematic programs or abstract-state traces.
Comments describe omitted construction or already discharged ordinary
obligations. No example assumes a stored reference, a returned reference,
an unsafe operation, or a new writer-visible wildcard syntax.

Verdicts compare:

- **P:** the three proposed rules, added to v0.60 with the other rules still
  enforced. A preservation claim does not silently override an independent
  invalidation or operation prerequisite. Where the prose has conflicting
  readings, both are stated instead of inventing a unique verdict.
- **W:** the explicit amendment in [RULES.md](RULES.md).
- **Bad completion:** a tempting additional interpretation under which a
  program is accepted incorrectly. This is evidence against that completion,
  not a claim that the three sentences unavoidably require it.

For local transfer cases, `p at R.**` in a comment is a premise that p is a
valid widened reference at that checkpoint. It is not source. Such cases
test the proposed transfer independently of C01's formation failure. A full
P program cannot silently bypass that failure to reach the checkpoint.

Common source declarations, reused unless a case says otherwise:

```wf
struct Node {
  tag: u64;
  next: Option<Box<Node>>;
}

struct Tree {
  tag: u64;
  left: Option<Box<Tree>>;
  right: Option<Box<Tree>>;
}

struct Branch {
  tag: u64;
  children: Slots<Box<Branch>, 8>;
}

fn read_tag(node: &Node) -> result: own u64 reads(node.tag) contract {
  ensures result == deref(node).tag;
} {
  return deref(node).tag;
}

fn needs_nonzero(node: &Node) -> result: own u64 reads(node.tag) contract {
  requires deref(node).tag != 0_u64;
} {
  return 1_u64 / deref(node).tag;
}

fn clear_next(node: &Node) -> result: own unit writes(node.next) {
  set deref(node).next = None<Box<Node>>();
  return unit;
}

fn replace_node(node: &Node, replacement: own Node) -> result: own unit writes(node) {
  set deref(node) = move replacement;
  return unit;
}

fn write_two(left: &u64, right: &u64) -> result: own unit writes(left), writes(right) {
  set deref(left) = 1_u64;
  set deref(right) = 2_u64;
  return unit;
}
```

`read_tag`'s field datum is a function-entry datum, as FN-9 requires.
The function only reads it, so its entry image remains stable. This is not
an invented general postcondition on the exit value of a mutable scalar.

## C01. The ordinary list walk still loses its refinement

```wf
fn last_tag(root: &Node) -> result: own u64 reads(root) {
  let p = root;
  loop {
    let link = &deref(p).next;
    match deref(link) {
      Some(value: child) => {
        set p = &deref(child).inner;
      }
      None() => {
        return deref(p).tag;
      }
    }
  }
  return 0_u64;
}
```

The final return supplies FN-1's conservative loop continuation; no claim
that the compiler proves this walk terminates is needed.

**v0.60: rejected** by REF-1's growing-path restriction. **P: rejected** on
the next iteration if REF-2 and ENT-3.S15 remain literal: p's new path used
the Some refinement and that fact is lost at the arm exit. The address is
actually still valid, so this is a safe program which P fails to enable.
**W: accepted.** The hidden path witness survives the arm exit; the next
iteration must match its new `next` anew. W does not retain the old Some
fact for a new selection.

## C02. A tree cursor can choose either child

```text
let p = root;                         // root: &Tree
loop {
  if choose_left {                    // an ordinary Bool input
    match &deref(p).left {
      Some(value: child) => set p = &deref(child).inner;
      None() => break;
    }
  } else {
    match &deref(p).right {
      Some(value: child) => set p = &deref(child).inner;
      None() => break;
    }
  }
}
let answer = deref(p).tag;
```

Here and in later `text` blocks, `match &place` abbreviates the v0.60
`let link = &place; match deref(link) { ... }` spelling of C01, and each arm
body is a braced block.

**P: rejected** for C01's refinement reason. If that omission is repaired,
its cone rule accepts both branches correctly. **W: accepted.** The
description covers arbitrary finite words over left and right, not just
repetitions of one discovered suffix. No tree-height or path-choice fact is
inferred. W's anchor is the entering Tree place.

## C03. Reset to the original root

```text
let p = root;
loop {
  if restart {
    set p = &deref(root);
  } else {
    // Match p.next and descend as C01, or break on None.
  }
  let value = deref(p).tag;
}
```

Ignoring the independent C01 issue, **P: accepted and correct only when
`**` includes the empty suffix. W: accepted.** If an implementation means
"strict descendant", its state after the reset is false. Treating p as
strictly below root could also wrongly separate p's tag from root's tag.
The reference can reset without moving or rewriting the root.

## C04. Two cursors may both read the same node

```wf
// p and q are valid, independently selected cursors under R.
let a = read_tag(node: p);
let b = read_tag(node: q);
```

**P: accepted; W: accepted; correct.** They may be equal, ancestors of one
another, or on different branches. None of those possibilities makes two
reads unsafe. Equality of a and b does not follow from the cone descriptors.

## C05. Two cursors can make primitive stores sequentially

```wf
// p and q are valid at R.** and may actually be equal.
set deref(p).tag = 1_u64;
set deref(q).tag = 2_u64;
let value = deref(p).tag;
```

**P: accepted; W: accepted; correct sequentially.** Neither store removes
a place. The final value may be 2, so a fact that it is still 1 must die
when the second store may overlap it. PAR must not execute these stores
concurrently merely because neither invalidates a pointer.

## C06. A link store invalidates the other cursor

```wf
// p and q are valid at R.**, with q possibly below p.next.
set deref(p).next = None<Box<Node>>();
let value = deref(q).tag;
```

**P: rejected; W: rejected; correct.** q can name a freed child. If the
actual q was in a different branch this refusal is conservative; the cone
has no proof of that separation. The first statement alone is allowed.

## C07. Current target and a saved descendant are different witnesses

```text
// p is valid at R.**; match its next into child.
let q = &deref(child).inner;
set deref(p).next = None<Box<Node>>();
let a = deref(p).tag;                 // okay
let b = deref(q).tag;                 // error
```

**P: rejects the whole fragment at q; W: rejects at q; correct.** p's
Node still exists and q's Node may not. W additionally handles a whole
`set deref(p) = move replacement` in the same way. Keeping p valid must
not keep all paths formed through its old contents valid.

## C08. A callee may replace the cursor's target

```wf
// p and q are valid at R.**; q is not an actual of this call.
replace_node(node: p, replacement: move fresh_node);
let a = deref(p).tag;
let b = deref(q).tag;
```

**P: rejects at q; W: rejects at q; correct.** With the last line removed,
both accept correctly. The single `writes(node)` protects p's existing
target and invalidates possibly destroyed descendants and unrelated
summary witnesses. The owned replacement's different root is checked in
EFF-5 too. No callee-body inspection is needed.

## C09. Sibling effects through one captured cursor

```wf
fn inspect_and_clear(node: &Node) -> result: own u64 reads(node.tag), writes(node.next) {
  let value = deref(node).tag;
  set deref(node).next = None<Box<Node>>();
  return value;
}

// p is valid at R.**.
let value = inspect_and_clear(node: p);
```

**P with every effect flattened to `R.**`: rejected**, a safe false negative.
Its two substituted entries overlap and one writes. **W: accepted.** Both
suffixes are relative to the same captured p, so ordinary field separation
proves them disjoint. This does not prove separation for `.tag` and `.next`
through two independent cursors, which might be ancestor and descendant.
P has to specify which of these substitution schemes it means.

## C10. Root read plus cursor write in one call

```wf
fn observe_and_clear(root: &Node, cursor: &Node) -> result: own u64 reads(root), writes(cursor.next) {
  let value = deref(root).tag;
  set deref(cursor).next = None<Box<Node>>();
  return value;
}

// p is valid somewhere under root.
let value = observe_and_clear(root: root, cursor: p);
```

**P: rejected; W: rejected; correct for this declared row.** The `reads(root)`
entry covers the entire root. Invalidating root or p after the call would
not discharge its pairwise conflict. This particular body could declare
the narrower `reads(root.tag)`; that changes the interface and can expose
additional ordinary known-prefix separation, but does not license ignoring
the row actually written here.

## C11. Root and cursor both read

```wf
fn observe(root: &Node, cursor: &Node) -> result: own u64 reads(root), reads(cursor) {
  let ignored = deref(root).tag;
  return deref(cursor).tag;
}

let value = observe(root: root, cursor: p);
```

**P: accepted; W: accepted; correct.** Overlap is permitted for read/read.
An owning root is not frozen just because a cursor exists. Reading an
affine Node whole without a reference or move is still not admitted.

## C12. Merely being an actual must not protect a cursor

```wf
fn replace_ignoring_cursor(root: &Box<Node>, unused: &Node, replacement: own Box<Node>) -> result: own unit writes(root) {
  set deref(root) = move replacement;
  return unit;
}

// owner: own Box<Node>; replacement: own Box<Node>, at a different root.
let cursor = &owner.inner;
replace_ignoring_cursor(root: &owner, unused: cursor, replacement: move replacement);
let value = deref(cursor).tag;
```

This complete local counterexample needs no wildcard or enum refinement.
The row has no effect on `unused`, so there is no root/cursor effect pair
to reject. Replacing the Box frees the old cell.

**P read with REF-2 and rule (2): rejected; correct. W: rejected.** A blanket
reading of EFF-5(3)'s "a reference that is itself an argument" exemption
**accepts incorrectly**, leaving a dangling reference. The exemption must
mean the actual's own content access, not immunity from other actuals.
The sentence "does not invalidate itself" supports the narrower reading;
this report does not assert an observed compiler defect. The same issue
applies to an unused widened cursor. Omitting the last line should be
accepted, with cursor invalid afterwards, rather than making unused
reference arguments themselves exclusive loans.

## C13. Different known subtrees retain useful separation

```text
// forest.left_root and forest.right_root are distinct Node fields.
// p is widened under forest.left_root, q under forest.right_root.
write_two(left: &deref(p).tag, right: &deref(q).tag);
```

**P: accepted; W: accepted; correct.** The anchor paths separate before
the wildcard. W must not widen both to `forest.**` just because their
ultimate owner is the same variable when their flows never join. A
coarser root-only design would be safe but would unnecessarily reject this.

## C14. Replacing the current enum removes its exposed refinement

```text
// link is a widened reference to Option<Box<Node>>.
match deref(link) {
  Some(value: child) => {
    set deref(link) = None<Box<Node>>();
    let value = deref(link).Some.value.inner.tag;
  }
  None() => { }
}
```

**P with inherited refinement kills: rejected; W: rejected; correct.** link
still names an existing Option slot, but that slot is now None. A version
which lets rule (3)'s "leaves p valid" preserve the Some fact **accepts
incorrectly** and reads absent payload storage. Sealing only a traversed
prefix, as W does, does not seal the current target's old contents.

## C15. A descended reference may survive its selecting arm under W

```text
let p = root;
loop {
  match &deref(p).next {
    Some(value: child) => {
      set p = &deref(child).inner;
      if stop_here { break; }
    }
    None() => { break; }
  }
}
let value = deref(p).tag;
```

**P: rejected; W: accepted; actually safe.** The child root has not ended
scope: only the local alias `child` and the selecting arm have ended.
W must retain the validated concrete ownership location without exposing
the lost Some fact. OWN-11's structural check does not authorize ignoring
this edge merely because the source breaks.

## C16. Changing a hidden owning enum must revoke its witness

```text
// After a walk, p may name a descendant reached through owner.inner.next.Some.
set owner.inner.next = None<Box<Node>>();
let value = deref(p).tag;
```

**P: rejected; W: rejected; correct.** This is precisely the physical
destruction that C15's harmless arm exit is not. If p's narrow anchor is
already below `owner.inner.next`, the write is an ancestor of its anchor
and is still caught. A rule testing only "write below R" **accepts
incorrectly** if it replaces, rather than supplements, REF-2.

## C17. Descending into a window of boxes

```text
// p: &Branch. Assume an ordinary guard proves i < deref(p).children.len.
set p = &deref(p).children[i].inner;
// Repeat within a loop, with a fresh bound at every descent.
```

**P: the path-shape idea accepts; its treatment of lost bounds is unspecified.
W: accepted** when formation checks every bound. The hidden path witness
retains existence of the selected element; it does not grant a bound for
the next node's children. Bounds checked once at the first node are
insufficient for any later node.

## C18. Removing or remaking a containing window

```text
// children: own Slots<Box<Node>, 8>, nonempty; p may be in children[0].inner.
// The selected slot may also be p's narrow loop-entry anchor.
let removed = take_back(window: &children);
let value = deref(p).tag;
```

Choose a one-element window. **P with OP-10/REF-2: rejected; W: rejected;
correct.** Even if the taken Box's allocation remains live in `removed`,
the old ownership path has disappeared and a move never reroots p.
Releasing `removed` then makes the address physically dangling too.
Replacing `take_back` with an admitted `remove_at` or `grow` on a containing
boxed window has the same invalidation conclusion. For a Ring, front
insertion or removal additionally changes the logical coordinates.
Treating `writes(window.len)` as a harmless scalar-field update while
ignoring the rest of the operation **accepts incorrectly**.

## C19. Index capture, and the precision cost of append

```text
let i = 0_u64;
// children has at least two elements; select a valid path.
let p = &children[i].inner;
set i = 1_u64;
let value = deref(p).tag;
```

**P: accepted; W: accepted; correct.** p still names the element selected
with zero. This is already REF-1 behavior, not a wildcard contribution.
For a widened cursor anchored at `children[0].inner`, an ordinary
`place_back` with room proved and the original slot bound retained can be
disjoint by WIN-2, so **both accept continued use**. If instead the cursor's
only cover is an enclosing Branch's whole subtree, an append somewhere
inside that cover is conservatively invalidating under P and W: **both
reject its subsequent use**, even if that particular append was harmless.

## C20. Safe swaps preserve their endpoints

```wf
// p is a valid widened Node cursor; spare is an independent own Node.
swap(first: p, second: &spare);
let a = deref(p).tag;
swap(first: p, second: p);
let b = deref(p).tag;
```

**P: accepted; W: accepted; correct.** The first call exchanges disjoint
complete places, the second exchanges the identical place. Old descendant
references can become invalid. A swap between two complete array slots
also remains accepted when the indices might be equal; same-type slot
equality is not an ancestor relation.

## C21. An ancestor and descendant are not a safe swap pair

```text
// owner.inner is Node A, whose next owns Box B containing a Node.
// p is a valid cursor that can select that child Node B.
swap(first: &owner.inner, second: p);
set deref(p) = Node(tag: 0_u64, next: None<Box<Node>>());
```

**P under OP-11's equal-place-only allowance: rejected; correct.
W: rejected.** Interpreting the swap exemption as "any overlapping pair of
the same type" **accepts incorrectly**. The first exchange puts the old
ancestor value, containing the Box that owns B, into B. B now owns itself;
it is no longer a descendant of the externally rooted Node A. A subsequent
replacement of B can release B while attempting to store into B. Even
without the second line the single-owner forest invariant has failed.
Checking validity only after the exchange does not repair the exchange.
The problem exists with a finite child path as well as with a wildcard.

## C22. Moving the root invalidates, but is not itself forbidden

```wf
// p is valid somewhere in owner.inner; owner: own Box<Node>.
let moved = move owner;
let value = deref(p).tag;
```

**P plus REF-2: rejected; W: rejected; correct.** The old path ceases to
exist even when the allocation address has not changed. Omitting the last
line is allowed. A version that tests only writes/moves below a narrow R,
and drops the inherited root rule, **accepts incorrectly**. A new reference
must be formed from moved if it is needed.

## C23. Scope end cannot be widened away

```text
let p = &outer.inner;
if condition {
  let temporary = box_new<Node>(value: make_node());
  set p = &temporary.inner;
  // A nested walk may widen p under temporary.inner.
}
let value = deref(p).tag;
```

`make_node()` abbreviates any ordinary owned constructor. **P: rejected;
W: rejected; correct.** The true edge releases temporary before the join.
The false edge's live outer target cannot make the true edge valid. The
rejection also holds without a loop. An anchor needs scope identity, not
just a type or an allocation class.

## C24. Use after a completed walk

```text
let p = root;
loop {
  if deref(p).tag == wanted { break; }
  // Match next; descend on Some, break on None, as C01.
}
let value = deref(p).tag;
```

**P: fails C01 unless its refinement rule is repaired. W: accepted.** The
continuation retains the union of all break-edge targets, including the
original root. It does not reset p to its loop-entry path. A post-loop
write via root that can destroy one of those targets invalidates p.
The loop proves neither that value equals wanted nor that a match was found.

## C25. Join across two roots

```text
let p = &a.inner;
if choose_b {
  set p = &b.inner;
}
// Descend p in a loop, as C01.
let moved = move b;
let value = deref(p).tag;
```

**P with REF-1's union: rejected; W: rejected; correct.** The summary must
include the b alternative. W records both root cones. A single-R
implementation that arbitrarily keeps a **accepts incorrectly**. The move
is outside the loop so OWN-11 is not the reason for the rejection.

## C26. A valid edge cannot repair an invalid edge

```text
// p is a valid widened cursor under owner.inner.
if replace_it {
  let fresh_node = make_node();
  let replacement = box_new<Node>(value: move fresh_node);
  set owner = move replacement;
}
let value = deref(p).tag;
```

**P with fact joins: rejected; W: rejected; correct.** The replacement is
constructed on the true edge; no outer owned donor is conditionally moved.
owner is live on both edges, so this is not a LIV-1 disagreement. One incoming
reference witness is nevertheless dead. Taking a union of location sets
while taking a union of validity facts **accepts incorrectly**. W's validity
join is conjunction, even if both paths have the same cone description.

## C27. Nested loops reuse a stable outer anchor

```text
let p = root;
loop {
  loop {
    // Descend p until a local condition holds, then break inner loop.
  }
  if done { break; }
  // Possibly descend p once more, then continue outer loop.
}
let value = deref(p).tag;
```

With checked selections and no destructive side effects, **W: accepted**.
**P: no complete algorithmic verdict beyond C01's issue.** If its fixed
point means retaining the original root cone, acceptance is correct. If
each recheck invents an anchor below the newest p, the checker need not
terminate or may lose earlier alternatives. `R.**.next.**` is not W's cover
representation. An inner loop is not a fresh owner lifetime.

## C28. Indirect self-extension through another reference

```text
let p = root;
let q = root;
loop {
  // Under a current Some refinement for q.next:
  set p = &deref(q).next.Some.value.inner;
  set q = &deref(p);
}
```

**P with "through itself" meaning only the written variable: unspecified,
and it cannot safely accept by finite unrolling.** The path grows around
the p/q cycle. A conservative implementation may reject it, losing a safe
walk. **W: accepted** when each selection is checked, because the positive
reference-flow cycle is summarized. Rebinding q does not mutate storage or
retarget independently saved aliases. Full source uses C01's matching form.

## C29. One recheck is not a fixed-point algorithm

```text
let p = &a;
let q = &a;
let r = &a;
loop {
  set r = &deref(q);
  set q = &deref(p);
  if restart_at_b {
    set p = &b;
  } else {
    // Checked descent from p, or leave the loop on None.
  }
  write_two(left: &deref(r).tag, right: &b.tag);
}
```

Take restart_at_b true for successive iterations; a and b are distinct
owned Nodes. After the third transfer, r can name b. The call then passes
one place as two independently written actuals, which EFF-5 forbids.

```text
Entering head:       p={a},       q={a},       r={a}
After first sweep:   p={a.**,b.**}, q={a},        r={a}
After one recheck:   p={a.**,b.**}, q={a.**,b.**}, r={a}
Next body sweep:     r receives q's b alternative; the call must reject.
```

The transfer order r, q, p makes this a real propagation delay, not a
choice of numeral syntax. This tests path propagation after C01's
independent refinement-loss problem has been repaired. Unrepaired P still
rejects the complete program because its structurally checked else branch
contains that descent, even when restart_at_b is always true at runtime.
**For the path analysis, P interpreted as "iterate to a true fixed
point": rejected; correct. Exactly one recheck after repairing C01:
accepted incorrectly. W: rejected.** A dependency-aware algorithm can
discover the conflict faster, but one universal number of body passes is
not justified. Longer chains give arbitrarily longer delays.

## C30. Root reads are free of invalidation, not of interference

```wf
// p is valid under owner.inner and can be the root Node.
let before = owner.inner.tag;
set deref(p).tag = 9_u64;
```

**P: accepted; W: accepted; correct sequentially. PAR: denied** when p
may be that root. An interpretation that omits root reads from the
footprint because "reads are free" **permits an incorrect execution**:
before could see 9 instead of the source-order value. Both pointers can
remain valid while that race changes the result.

## C31. Parallel work on separated anchors

```text
// left and right are distinct owned roots (or separated struct fields).
clear_next(node: p);                 // p under left.**
clear_next(node: q);                 // q under right.**
```

**P: accepted; W: accepted; PAR-1 permitted** subject to the ordinary
operand, result, control, and storage-retention premises. Distinct anchor
subtrees remain disjoint at arbitrary depth because values contain only
owned values. A cone need not conservatively cover every heap object.

## C32. A loop-carried cursor is not a counted partition

```text
let p = root;
for i in 0_u64..count {
  set deref(p).tag = i;
  // Checked descent or reset of p.
}
```

**P: subject to its formation omission; W: sequentially accepted** with
valid selections and normal ownership. **PAR-2: denied.** The write is not
one of PAR-2's direct affine element forms or established range partitions,
and p is loop-carried dataflow. The fact that a particular walk visits
different nodes does not furnish that proof. A separate helper called on
each existing permitted disjoint range may walk owned children below that
range; the range proof, not a guessed cursor distance, supplies separation.

## C33. Rebinding kills a contract fact about the old referent

```text
// p names a Node of tag 1; its next child has tag 0.
let observed = read_tag(node: p);
if deref(p).tag != 0_u64 {
  // Match next and rebind p to the child, as C01.
  let value = needs_nonzero(node: p);
}
```

**P with ENT-5 holder support: rejected; W: rejected; correct.** After
rebinding, p can name the zero node. The old returned number observed is
still 1; its equality to the new `deref(p).tag` is not. Keeping a fact
because the before/after cone is R.** **accepts incorrectly**, admitting a
zero denominator. This failure is about target/value identity, not about
whether the new pointer is valid.

## C34. A primitive write preserves the cursor but kills its field facts

```wf
// p at owner.inner.** can currently name owner.inner itself.
if deref(p).tag != 0_u64 {
  set owner.inner.tag = 0_u64;
  let value = needs_nonzero(node: p);
}
```

**P plus ENT-5: rejected; W: rejected; correct.** If p names the root, its
field is now zero. The primitive exception is only a validity exception.
Extending it to fact preservation **accepts incorrectly**. There is no
need to invalidate p itself: a fresh read and branch can establish a new
fact about its new content.

## C35. Two R.** descriptions must not be one proof term

```wf
// p and q are independent valid cursors at R.**.
// Concrete state: deref(p).tag == 1, deref(q).tag == 0.
if deref(p).tag != 0_u64 {
  let value = needs_nonzero(node: q);
}
```

**P with ENT-2 retained: rejected; correct.** ENT-2 explicitly keeps
different canonical source spellings as different terms even when their
storage overlaps; no fact establishes q's requirement. If an implementation
replaces that rule with resolved-summary interning and identifies both
fields as the same `R.**.tag` term, it is **accepted incorrectly** and
divides by zero. **W: rejected**, preserving ENT-2's source identities and
keeping separate target metadata for overlap. A may-alias summary cannot
serve as a must-alias theorem. This counterexample protects an existing
rule; it is not a new unsoundness forced by P.

## C36. Target evaluation does not bypass right-hand-side revalidation

```wf
fn replace_and_number(root: &Box<Node>, replacement: own Box<Node>) -> result: own u64 writes(root) {
  set deref(root) = move replacement;
  return 7_u64;
}

// p is valid inside owner.inner.
set deref(p).tag = replace_and_number(root: &owner, replacement: move replacement);
```

**P plus SET-1: rejected; W: rejected; correct.** The target address is
evaluated before the call; the call can free it. The final primitive store
cannot use the primitive exception to restore a reference already killed
by the right-hand side. A checker that validates p only at target
evaluation **accepts incorrectly** and stores through a dangling address.

## C37. Removal through a link slot is expressible without a hole

```wf
fn remove_head(link: own Option<Box<Node>>) -> result: own Option<Box<Node>> pure {
  match link {
    Some(value: cell) => {
      let node = move cell.inner;
      let Node(tag: old_tag, next: tail) = move node;
      return move tail;
    }
    None() => {
      return None<Box<Node>>();
    }
  }
}

// link is a valid widened reference to an Option<Box<Node>> slot.
set deref(link) = remove_head(link: move deref(link));
```

**P with OP-12: accepted at this checkpoint; W: accepted; correct.** The
slot stays initialized across the atomic update; on Some the removed cell
is consumed and the tail becomes the new slot value. link continues to
name the slot, not the removed Node. Other descendant cursors die.
The loop which first searches for that slot still needs C01's repair.
Splitting the operation into a standalone move through the reference and
a later assignment is **rejected by both**; wildcard paths do not add holes.

## C38. A cursor can leave its first anchor only by a new checked formation

```text
// Initially p is under owner.inner.left.Some.value.inner.
loop {
  if restart {
    set p = &owner.inner;            // an ancestor outside the first cone
  } else {
    // Checked descent from p.
  }
}
```

**P: no sufficient single-anchor join rule is stated.** Retaining only the
first narrow R would accept with a false descriptor. A conservative
implementation must reject or enlarge it. **W: accepted**, with the anchor
shortened to a common containing prefix; any exposed old target facts die.
Resetting instead to another owned root creates a second cone, as C25.

## C39. A readonly prefix cannot disappear inside the wildcard

```text
struct Holder { readonly tree: Tree; }
let p = &holder.tree;
loop {
  // Checked left or right descent from p, as C02.
  set deref(p).tag = 0_u64;
}
```

**P plus TYPE-2/SET-1: rejected; W: rejected; correct.** Every descendant
is reached through the readonly field. Losing that prefix's permission
when normalizing to a typed Tree cursor **accepts incorrectly**. The same
requirement applies to a read-only formal row, named const roots where
types permit them, and a join with a read-only alternative. W records the
permission separately from the target type and cone.

## C40. Widening never proves a new payload selection

```wf
// p is valid at R.**, with no current refinement of deref(p).next.
set p = &deref(p).next.Some.value.inner;
```

**P plus REF-1: rejected; W: rejected; correct.** A node with next None
refutes the missing formation premise. An informal example omitting its
match is not evidence that the operation becomes total. Likewise an
unguarded `children[i].inner` is rejected under both schemes.

## C41. Aliasing a cursor does not rebind the alias

```text
// p is valid at R.**.
let q = p;
// Checked descent and rebinding of p, as C01.
let old_value = deref(q).tag;
let new_value = deref(p).tag;
```

**P with REF-1 alias snapshots: accepted at this checkpoint; W: accepted;
correct.** q still names the old node. A rebinding changes a name, not
storage. For the variant without the descent, `set deref(q).next = None`
followed by `deref(p).tag` is **rejected conservatively by P(3)**, which
invalidates every other reference, and **accepted by W**, which knows
q and p name the same target and this is a content write for both.

## C42. A scalar effect and an aggregate effect are not interchangeable

```text
// p and q independently summarize one subtree.
set_leaf(node: p);                   // declared writes(node.tag)
let a = deref(q).tag;
replace_node(node: p, replacement: move fresh);
let b = deref(q).tag;
```

`set_leaf` is an ordinary helper that assigns that u64 field and returns
unit. **P and W accept through a and reject b; correct.** The first row
can use the primitive-leaf validity rule, while still killing q's tag
facts. The second row permits structural replacement regardless of whether
one particular callee body happens to replace it with an equal value.
Actual spelling and a currently convenient implementation cannot narrow
the declared boundary.

## What these cases establish

The actual unsafe continuations above have concrete states: a freed Box
cell (C12/C16/C36), an absent payload (C14), an ownership cycle (C21), a
forbidden aliased write pair (C29), an incorrect parallel observable (C30),
or a false arithmetic-domain proof (C33-C35). They refute the named bad
completion. They do not show that an implementation retaining all the
ordinary WF checks must accept those programs.

The positive cases establish the intended scope of W by reasoning, not by
an implementation result. In particular, C01/C15 require an explicit
semantic amendment about hidden refinements. None of these documents
claims that today's compiler implements W or that these cases constitute
a conformance suite.
