# Incremental ownership design cases

These cases preserve the local ownership discussion through branch joins,
conditional cleanup, and fixed-object loops with early exits, recorded on
2026-09-15 and 2026-09-16, with source-form and decomposition notes added on
2026-09-17. They are hand-derived expectations under candidate
semantics, not WF source, compiler tests, measured results, or an
adopted language decision. The consumer is the [ownership investigation](RESEARCH.md).
Keep this catalog current while it guides discussion and later experiments;
supersede individual cases with linked executable witnesses when those preserve
their premises and distinguishing behavior. Do not lose the rejected variants.

Record each completed discussion round here before introducing another feature.
Retain stable case IDs. If a rule or expected result changes, update the affected
cases and state the reason; a previous expectation is not a constraint on the
final language. Start with permissive operations and consider restrictions when
a concrete failure or excessive complexity gives them a purpose.

## Scope and notation

The current scope is one sequential procedure with finitely many independent,
fixed objects of known origin. Their contents are ordinary copy integers. Local
locators can be copied and rebound. The branch layer adds Boolean conditions,
Boolean assignment/negation, and `if`/`else`. The loop layer adds the explanatory
form `repeat n times`, where n is a finite nonnegative runtime count evaluated
at loop entry, and `break` for leaving the enclosing loop. Objects are created
before the loop; ordinary scalar and locator
temporaries inside its body do not create abstract allocations. There are no
calls, callbacks, fields, containers, later allocations, storage reuse, or concurrency.
Conditions may be unknown at compile time. No unknown incoming reference is
introduced as a counterexample to known local origins.

`a = object(10)` creates a live, initialized abstract slot A with one recorded
disposal obligation. `a` names its owner; `p = ref(a)` creates a locator for A.
Locators name storage, not a value that follows a later move. Ordinary scalar
temporaries such as a taken integer do not create another abstract allocation
obligation. `move` denotes value transfer, not settled WF surface syntax.

| Operation | Premise in this scope | Result |
|---|---|---|
| `ref(a)`, copy/rebind locator | Valid source binding; rebinding does not silently discard an owned resource | Captured target is copied; earlier copies retain their targets. |
| `read(p)` | Target live and initialized | Read the copy value. |
| `write(p, n)` | Target live, compatible copy scalar | Initialize or overwrite the slot; invalidate old current-value facts. |
| `old = replace(p, n)` | Target live and initialized | Return old value and install new same-typed value in the same slot. |
| `v = take(p)` | Target live and initialized | Return content; slot stays live but empty. |
| `put(p, v)` | Target live and empty | Fill the slot. This deliberately narrow name does not restrict ordinary scalar `write`. |
| `release(a)` or `release(p)` | Target live, its disposal authority available in the current local context, and content obligations discharged | Consume that one obligation and permanently end the slot. |

The working variant permits release through a locator: it identifies the target
whose obligation is consumed, without copying an obligation into each locator.
An owner-only release interface remains an alternative, not a demonstrated
safety necessity in this fragment. Interface authorization outside the local
context is not yet designed. General affine/linear content disposal and automatic
scope cleanup remain separate choices. There is no requirement to restore a hole
before releasing its empty storage.

Fragments omit unrelated cleanup unless exit obligations are the subject. Each
`REJECT` line marks the intended first failing operation; a variant replaces that
line or suffix rather than executing through a rejection. A pending obligation
is not by itself an invalid access or proof that the language must require
explicit release at exit.

The [earlier executable probe](../../experiments/access-state/RESULTS.md#local-baseline-results)
has two objects/two locator slots, owner-only release and explicit linear cleanup.
It does not implement these branch/loop cases or the permissive locator-release
variant. Its `Store` covers scalar write/initialization rather than a distinct
empty-only `put`. Its successful run is evidence only for its stated rules.

## Straight-line cases

### S01: Sequential aliases and stable copies

```text
a = object(10)
b = object(20)
p = ref(a)
q = p
write(p, 11)
read(q)                   // ACCEPT: 11
p = ref(b)
write(p, 21)
read(q)                   // ACCEPT: 11, still A
read(p)                   // ACCEPT: 21, now B
```

No exclusive interval is needed for these sequential scalar accesses. Rebinding
changes a locator binding; writing changes the target shared by its aliases.

### S02: Replacement preserves storage and transfers the old value

```text
a = object(10)
p = ref(a)
q = p
old = replace(p, 20)
read(q)                   // ACCEPT: 20
read(a)                   // ACCEPT: 20
read(old)                 // ACCEPT: 10
```

The target identity remains A. Old value-dependent facts about A expire.
This does not establish preservation for references into future subobjects.

### S03: A hole is a property of the target, not one locator

```text
a = object(10)
p = ref(a)
q = p
v = take(p)
read(q)                   // REJECT: A is empty
```

Replacing the last line with `read(a)` or another `take(q)` also rejects. The
owner name and another alias cannot bypass the same initialization state.

### S04: Another alias can restore the hole

```text
a = object(10)
p = ref(a)
q = p
v = take(p)
put(q, move v)
read(p)                   // ACCEPT: 10
```

p and q continue to target A, never the new location holding v. Copying the
locator while A is empty also works; copying does not read A's contents.

### S05: A hole need not be restored before its storage ends

```text
a = object(10)
p = ref(a)
v = take(p)
release(a)                // ACCEPT: release the empty slot
read(v)                   // ACCEPT: 10
```

The moved-out value is independent of the old slot. Empty-slot cleanup must not
dispose of that value a second time. Non-copy content obligations are deferred.

### S06: Replacement requires old content at its commit

```text
a = object(10)
p = ref(a)
q = p
v = take(q)
old = replace(p, move v)  // REJECT: there is no old value to return
```

Replacing the last line with `put(p, move v)` accepts. Operand evaluation that
performs a take must not leave an earlier initialization premise authoritative.

### S07: Release is permanent and affects every access path

```text
a = object(10)
p = ref(a)
q = p
release(p)                // ACCEPT in the permissive variant
read(q)                   // REJECT: A has ended
```

Separate variants replace the last line with `read(a)`, `write(q, 7)`, `take(q)`,
`release(q)`, or `release(a)`; all reject. A write cannot revive dead storage,
and cleanup cannot release it again. Copying or discarding an inert locator is
allowed without accessing A. Later allocation/reuse is not part of this case.

## Branch cases

### B01: One locator can have alternative targets

```text
a = object(10)
b = object(20)
if cond { p = ref(a) } else { p = ref(b) }
read(p)                   // ACCEPT: both possible targets are initialized
```

The analysis represents both executions, not one guessed runtime choice.

### B02: Target and initialization must remain correlated

```text
a = object(10)
b = object(20)
if cond {
    p = ref(a)
    old = take(b)
} else {
    p = ref(b)
    old = take(a)
}
read(p)                   // ACCEPT: the selected target is always initialized
```

Swap the take targets so each arm takes p's selected object: the final read
rejects in both arms. Independent target/initialization sets cannot distinguish
these cases. The integers bound to old can be discarded in this copy-only scope.

### B03: A write initializes the selected target, not its entire may-set

```text
a = object(10)
b = object(20)
old_a = take(a)
old_b = take(b)
if cond { p = ref(a) } else { p = ref(b) }
q = p
write(p, 9)
read(q)                   // ACCEPT: 9
read(a)                   // REJECT: A may still be empty
```

Replacing the last line with `read(b)` also rejects. Exactly one object was
initialized; q's captured equality with p identifies that object.

### B04: A copied uncertain target supports take and restoration

```text
a = object(10)
b = object(20)
if cond { p = ref(a) } else { p = ref(b) }
q = p
v = take(p)
put(q, move v)
read(a)                   // ACCEPT: 10
read(b)                   // ACCEPT: 20
```

Between take and put, reading a or b unconditionally rejects. Restoration acts
on the same selected object; afterward both original slots are initialized.

### B05: Correlated distinct targets survive selected reclamation

```text
a = object(10)
b = object(20)
if cond {
    p = ref(a)
    q = ref(b)
} else {
    p = ref(b)
    q = ref(a)
}
release(p)
read(q)                   // ACCEPT
release(q)                // ACCEPT: both obligations are now consumed
```

After the first release, an unconditional `read(a)` or `release(a)` instead
rejects: A may be dead. The actual pairs are (A, B) and (B, A), not their
Cartesian product. The remaining obligation belongs to q's target.

### B06: Rebinding must update relations without retargeting saved copies

```text
a = object(10)
b = object(20)
if cond {
    p = ref(a)
    q = ref(b)
} else {
    p = ref(b)
    q = ref(a)
}
saved = p
p = q
release(p)
read(saved)               // ACCEPT: the original p target survives
read(q)                   // REJECT: now p and q have the same target
```

The old inequality between p and q cannot remain attached to their variable
names after assignment. The saved target remains distinct from their new target.

### B07: A conditional hole prevents reading but not scalar overwrite

```text
a = object(10)
p = ref(a)
if cond { old = take(p) }
read(p)                   // REJECT: A may be empty
```

Replace the last line with `write(p, 20); read(p)`: accept, returning 20 on both
paths. Ordinary copy-scalar write handles live empty and live full slots with
the same store. Empty-only `put` would lack its premise on the full path;
`replace` would lack old content on the empty path. Those operation-specific
premises must not unnecessarily narrow ordinary scalar assignment.

### B08: Repeating an unchanged condition recovers a path's state

```text
a = object(10)
p = ref(a)
if cond { old = take(p) }
if cond { put(p, 20) }
read(p)                   // ACCEPT: 20 if true, 10 if false
```

cond is not modified between tests. The second true edge implies the earlier
take occurred; the second false edge implies it did not.

### B09: Conditions refer to captured values, not permanent variable names

```text
a = object(10)
p = ref(a)
if cond { old = take(p) }
cond = !cond
if cond { write(p, 20) }
read(p)                   // REJECT: the original true path remains empty
```

Replacing the second conditional with `if !cond { put(p, 20) }` accepts. Boolean
assignment invalidates reuse of the old variable value, but its known relation
to the new value can preserve useful information. Forced loss of every relation
on assignment would be an approximation, not a semantic necessity here.

### B10: Join must not erase conditional disposal obligations

```text
a = object(10)
b = object(20)
if cond { release(a) } else { release(b) }
```

| Condition | A | B | Outstanding obligation |
|---|---|---|---|
| true | Dead | Live | B |
| false | Live | Dead | A |

Intersecting the two remaining-owner sets yields the empty set and is incorrect:
each execution still has one obligation. Explicit-consumption checking would
mistakenly consider it complete; automatic cleanup would mistakenly omit it.

### B11: A locator can identify the conditionally remaining obligation

```text
a = object(10)
b = object(20)
if cond {
    release(a)
    remaining = ref(b)
} else {
    release(b)
    remaining = ref(a)
}
read(remaining)           // ACCEPT
release(remaining)       // ACCEPT: neither obligation remains
```

The branches need not retain the same named owner. remaining designates the
live target whose obligation remains. The ordinary selected runtime pointer is
enough for the operation; the relationship to the obligation is static evidence.

### B12: Two conditional releases separate safety from completion

```text
a = object(10)
if c { release(a) }
if d { release(a) }
```

| c | d | Result |
|---|---|---|
| false | false | No release; obligation still pending. |
| true | false | Exactly one release. |
| false | true | Exactly one release. |
| true | true | REJECT at the second release. |

Independent unrestricted c and d cannot justify the second release. Avoiding
double consumption requires `not (c and d)`; completing this explicit release
sequence also requires `c or d`. Both hold when exactly one is true. In
particular, `if cond { release(a) }; if !cond { release(a) }` accepts if cond is
unchanged. A pending obligation's treatment at exit depends on cleanup policy.

### B13: Conditional scope cleanup is a policy choice with known input state

```text
a = object(10)
if cond { release(a) }
// Leave the owning scope.
```

At exit, the obligation is absent on the original true edge and present on the
original false edge. Explicit-consumption policy would require the latter to
be discharged in source. Automatic cleanup policy would release A only on the
latter path, logically `if !original_cond { release(a) }`.

These are alternatives, not two simultaneous acceptance rules. Later assignment
to cond cannot change which path needs cleanup. Cleanup placement on CFG edges,
retaining a condition, and other lowerings have not been compared. This scalar
model does not establish that cleanup needs a runtime flag or a machine action.
Any future generated cleanup must implement the selected cleanup semantics;
it must not repair an unproved access with a runtime safety test.

## Fixed-object loop cases

### L01: Take and restoration preserve the next iteration's entry condition

```text
a = object(10)
p = ref(a)
q = p
repeat n times {
    v = take(p)
    put(q, move v)
}
read(p)                   // ACCEPT: 10, including when n is zero
```

At the loop head A is live and initialized, p and q designate A, and A's
disposal obligation remains. Take temporarily empties A; put restores it before
the backedge. Entry establishes these relations and one body execution preserves
them. The hole inside the body is allowed. No concrete value of n or unrolling
of all iterations is needed for this argument.

### L02: An empty exit state becomes the next iteration's input

```text
a = object(10)
p = ref(a)
repeat n times {
    v = take(p)
}
```

For unrestricted n, reject: n can be at least two, and the second take accesses
an empty slot. Reusing the pre-loop initialized state independently for each
iteration is unsound. If n is known to be at most one, there is no repeated-take
error, but a post-loop read is not justified for both possible counts: zero
leaves A full and one leaves it empty. Storage remains live in both cases and
its disposal obligation remains. This is an expected semantic distinction, not
a claim that a particular numeric loop analysis has been implemented.

### L03: Relative initialization can be invariant while physical roles alternate

```text
a = object(10)
b = object(20)
old_b = take(b)
p = ref(a)
q = ref(b)
repeat n times {
    v = take(p)
    put(q, move v)
    tmp = p
    p = q
    q = tmp
}
read(p)                   // ACCEPT: 10
```

The reference/initialization component of the state at the loop head has two
possibilities, without enumerating counter values or complete execution histories:

| Head state | p | q | A | B |
|---|---|---|---|---|
| First | A | B | Full | Empty |
| Second | B | A | Empty | Full |

Both slots are live and both disposal obligations remain in either state.
Every iteration takes the full p target, fills the empty q target, and exchanges
the locators. It maps the first head state to the second and the second to the
first. The zero-iteration path also satisfies the same relation.

The invariant is relative: p and q designate distinct live slots, p's target
is full, and q's target is empty. It is not a requirement that A always be full,
that B always be empty, or that each object's entry state be restored each round.
A post-loop `read(q)` rejects; unconditional `read(a)` or `read(b)` also cannot
be justified for arbitrary n. `release(p); release(q)` accepts and consumes both
obligations, including the empty slot's obligation. The copied integer old_b is
independent of B's later contents.

### L04: A release on a break edge need not preserve the loop-head state

```text
a = object(10)
p = ref(a)
repeat n times {
    if stop {
        release(p)
        break
    }
    read(p)               // ACCEPT: the release path cannot reach this read
}
```

The body is safe: every reachable backedge retains live initialized A and its
unconsumed disposal obligation. The break edge instead carries dead A and the
consumed obligation. Requiring the loop-head invariant on that break edge would
unnecessarily reject the fragment.

The zero-iteration and normal-completion exits retain live full A; the break
exit has dead A. An unconditional post-loop `read(p)` or `release(p)` rejects
when both kinds of exit are possible. The safe fragment's pending obligations
on some exits are distinct from an illegal access; exit cleanup policy is B13.

### L05: A checked Boolean relation can guard later iterations after release

```text
a = object(10)
p = ref(a)
active = true
repeat n times {
    if active {
        read(p)
        if stop {
            release(p)
            active = false
        }
    }
}
if active {
    release(p)
}
```

Accept, with exactly one completed release including the zero-iteration case.
At the loop head, active implies live initialized A and a pending obligation;
not active implies dead A and a consumed obligation. Later iterations may
continue with a dead locator because they no longer access its target.

active is an ordinary source Boolean, not intrinsic evidence or a permission
token. Removing `active = false` makes later iterations or final cleanup unsafe
on the release path. Inserting `read(p)` between `release(p)` and the assignment
also rejects, although active is still true there: release has invalidated the
old state implication. The loop-head relation need not hold between those two
instructions, and it cannot be used as a timeless fact.

The Boolean branch is explicitly written runtime control, whose connection to
resource state must be proved. The checker does not insert a liveness test,
assume the variable name proves anything, or require every locator to carry
such a flag. This is an optional source form rather than a selected lowering.

### L06: A hole may leave through break when the continuation only releases it

```text
a = object(10)
p = ref(a)
repeat n times {
    v = take(p)
    if stop {
        break
    }
    put(p, move v)
}
release(p)                // ACCEPT: A is live on every exit
```

Normal completion and zero iterations leave A full; break leaves A empty.
All exits retain live A and its one disposal obligation, so release accepts.
Inserting an unconditional `read(p)` before release rejects because the break
exit is empty. There is no obligation to fill the hole merely to execute break.
The taken value v is a discardable integer in this layer; non-copy value cleanup
would require its own accounting and is not established by this example.

## Source-form exploration (2026-09-17)

This records the discussion following L06. It adds no operation or accepted
WF syntax, and does not change the 26 numbered cases above. The question is
whether the writer and checker can use the same visible state transitions.

Three forms remain open: ordinary source with a checker-derived state view;
explicit branch results and loop-carried values with entry contracts; or fully
named control-flow blocks with explicit state transitions. The second is a
candidate for the next source-form comparison, not an adopted requirement.

Joint branch results make the correlation in B05 visible:

```text
(p, q) = if cond {
    yield ref(a), ref(b)
} else {
    yield ref(b), ref(a)
}
```

Both results come from the same arm. The checker must still retain that
relationship; tuple-like syntax alone does not prove disjointness.

L03 can expose the roles carried between iterations:

```text
a = object(10)
b = object(20)
old_b = take(b)
(p_end, q_end) = repeat n carrying (p = ref(a), q = ref(b))
    state { Full(p); Empty(q); Different(p, q) }
{
    v = take(p)
    put(q, move v)
    next(q, p)
}
read(p_end)
release(p_end)
release(q_end)
```

The state clause is a checked entry contract, not a permanent property of a
copied reference. Entry must establish it; each `next` simultaneously assigns
the next iteration's roles and must reestablish it. `next(p, q)` instead fails:
the proposed full role is empty and the proposed empty role is full. Zero
iterations return the initial roles. Existing disposal obligations are retained
even though the abbreviated state clause does not spell them out.

Named control-flow exits could similarly distinguish an initialized live result
from an already released path. Such continuation labels do not by themselves
require a runtime enum. Storing a choice as an ordinary first-class value is a
separate representation question. A state view should display the facts used
by acceptance, not a second independent approximation. No improvement in
checking complexity or measured speed has been established by these spellings.

## Decomposition frontier (2026-09-17)

The subsequent external conversation and the gap review appended to
[the candidate comparison](COMPARISON-CORE2.md) motivate smaller experiments,
not a winner among the full candidates. In particular, that review reports
missing rules and identifies the comparison's timing figures as inherited
measurements of the existing proof engine, not measurements of the candidate
checkers. Candidate-specific cost claims are not lower bounds on every possible
ownership analysis.

Separate the questions an operation needs answered:

- Which storage instance does this expression select?
- Does that instance still exist, and does the selected part contain a value?
- Who carries the outstanding responsibility to dispose of it?
- Does an operation transfer content, transfer that responsibility, or end
  storage? These need not be the same event.

Apply the distinction between directly known, provable, and unresolved facts
to each question, not to an entire pointer or language feature. B04 already
shows that an unknown concrete target does not preclude proving safe take/put
through that same captured target. Uncertain state does not itself force a
runtime tag: B07's scalar overwrite and L06's release have premises true for
every remaining state. This does not establish support for arbitrary stored
pointers or arbitrary predicates.

The next small boundary is a single scalar allocation. First distinguish an
owner binding, its separately allocated storage, and a locator for that storage.
Compare transferring ownership with relocating the content and ending storage.
Then add scope exit, conditional ending, and address reuse one at a time.
Allocation failure, variable numbers of allocations, reference fields, containers,
and concurrency remain separate extensions. No heap surface type or lowering
has been chosen.

A large fixed backing array is a useful allocator thought experiment, provided
the contract also represents the start/end of each logical allocation inside it.
The continued existence of backing bytes alone does not establish access through
an old allocation reference. Conversely, direct access to still-initialized
array cells is not automatically a dangling-pointer error: the exposed interface
and the claimed equivalence must be stated. The experiment must distinguish
physical storage, initialized contents, and logical allocation validity.

## Working conclusions and unresolved boundaries

- In the known-origin straight-line fragment, target/state propagation can be
  exact without knowing machine addresses. In the branch fragment, analyze a
  constrained set of possible complete states. Cover every reachable state;
  extra approximated states may cause rejection but must not be ignored.
- Useful correlations include equal/distinct captured targets, selected target
  versus initialization/liveness, and target versus unconsumed obligation.
  Stored current-value facts depend on state; obligations cannot be discarded
  simply because they differ between predecessors.
- Exact finite sets of resource states are a semantic reference for these
  examples, not a selected compiler representation, a general cost bound, or
  permission to use an SMT solver for WF acceptance. Compact representations and supported
  deterministic derivations remain research questions.
- Read/write access modes are not needed for alias exclusion in this sequential
  fragment. The analysis still distinguishes operations and their state changes.
  No conclusion here removes later interface restrictions or parallel effects.
- Permissive locator release is the current exploration variant. Owner-only
  release, explicit versus automatic cleanup, and general affine/linear value
  handling are not settled. The earlier executable probe's stricter choices
  must not silently become language requirements.
- A loop-head relation must hold initially, justify each operation in the body,
  and hold again on every reachable backedge. Intermediate states may contain
  holes. L03 preserves a relation while changing each named object's state.
  These hand-derived invariants are not yet a selected inference algorithm or
  writer-facing invariant syntax.
- Backedges must reestablish the next iteration's premises. Break edges carry
  their actual state to the continuation instead. Subsequent operations must
  cover all reachable exits, including zero iterations; there need not be one
  common initialization or liveness state for every exit.
- Source forms exposing states and transitions are recorded above as alternatives.
  The next semantic boundary separates storage lifetime, content transfer, and
  ownership transfer for one scalar allocation. Calls, general dynamic allocation,
  relocation/reuse, structs, arrays, general object
  invariants, callbacks and concurrency remain outside these cases. Later
  features must replay the applicable cases and name any premise or expected
  result they change.
