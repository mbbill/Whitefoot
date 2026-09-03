# Current Plan: finish source-carried proof in the compiler

Status: IMPLEMENTED AND ACTIVATED as v0.40 on
`codex/source-proof`.

Active language authority: v0.41,
`899437ecf48691b9bc436c86a56ccc2a47fc4eb9290d546010296db7808c5761`,
which respells the six integer comparisons as symbols and delimits call-site
type application with `::` over the v0.40 proof surface this plan delivered.
`spec/kernel-spec.md` carries those exact ACTIVE bytes; the superseded v0.40 is
archived at `spec/kernel-spec-v0.40.md` and the merge-time record is in
`governance/APPROVALS.md`. Activation is branch content: nothing merges to
`main` until the owner approves the exact revision and canonical `make check`
passes on that revision. This document records technical direction and
sequencing; it grants no permission and adds no workflow gate.

## Outcome

Whitefoot is a proof-carrying systems language for AI-written, human-approved
code. “Proof-carrying” means the `.wf` source carries the contracts,
invariants, and finite proof steps needed to justify the program. The compiler
checks that source as part of ordinary semantic compilation and erases the
proof-only syntax before lowering.

The target bargain is unusually strong:

- an accepted program may contain a logic error, but no supported operation may
  execute memory corruption, a data race, an uninitialized read, silent
  overflow, an out-of-bounds access, or another unproved partial operation;
- the same checked ownership, effect, bounds, layout, address, and algebraic
  facts may remove checks, authorize optimizations, and establish `par`
  permissions without adding runtime proof machinery; and
- a supported partial operation is proved before emission or the source is
  rejected. The compiler never substitutes a writer trap, hidden check,
  impossible-case return, or other executable fallback for missing proof.

The cost is harder source. Relations that a human-oriented language leaves
implicit sometimes need an invariant or short proof script. Whitefoot accepts
that cost because the intended writer is AI: AI may search, construct
intermediate relations, and repair diagnostics, while the compiler remains the
small trusted checker and the human approves the requirements and result.

## Determinism boundary

Acceptance is a function of the complete source and the exact language/compiler
version. The official compiler uses no SMT solver, solver seed, randomized
ordering, heuristic proof search, timeout, or cumulative proof-work budget.
Machine speed and load cannot change accepted into rejected or the reverse.

This does not mean every accepted proof is one local table lookup. It means
that each automatic rule family has a specification-fixed finite domain and is
run to its specified completion. Fixed structural source ceilings are language
rules. Once an input is within them, the compiler finishes the required work;
it does not turn elapsed time into a language verdict. A successful query may
stop at its first witness in the fixed order because no later candidate can
revoke that success. An unproved result is returned only after the specified
family is exhausted.

External parser/finalizer resource exhaustion may stop compilation as a
non-language resource failure. It never means that a proposition was unproved,
that invalid source was valid, or that valid source was invalid.

## The author-visible automatic boundary

For every affine goal, AUTO is complete for exactly these routes, in this
specification order:

1. the zero-premise direct route;
2. every available published affine premise once, with coefficient one;
3. every unordered pair of available published affine premises, coefficient
   one for each, including one premise paired with itself; and
4. the final fixed L0-image route over the current difference-bound state.

The compiler does not select an undocumented “best” subset. If none succeeds,
AUTO is finished. The author can therefore decide from the language rules,
without probing compiler behavior, whether an explicit proof is needed.

The intended cut is:

- direct consequences and common consequences using one or two published
  affine premises are automatic;
- combinations requiring three or more published affine premises outside the
  final fixed L0-image route, special elimination routes, or future named
  nonlinear rules are directed by explicit `use` steps.

A later specification may deliberately strengthen AUTO. Within one exact
version, however, its boundary is fixed. A nonempty `use` block is a source
error when that version's AUTO already proves the target; redundant proof
scripts do not become a second canonical spelling.

## Source forms

`requires` is proved by every caller before argument transfer. `ensures` is
proved at every selected normal return and publishes the verified callable
summary to later callers. A recursive component cannot bootstrap itself from a
summary that has not been proved.

The word `invariant` always means “this relation holds at this source point.”
Placement supplies the extra control-flow meaning:

- a loop-header invariant is an induction contract;
- a body or ordinary-block invariant is a one-time program-point fact.

The complete `weigh` shape shows how a function postcondition, a loop contract,
and ordinary operation goals connect. A counted header carrying an invariant is
multiline and has no trailing comma:

```wf
fn weigh['w](weights: &'w buffer<u8>, count: own u64) -> total: own u32
    reads(weights) contract {
  define room = len(deref(weights));
  requires count <= room;
  requires count <= 1000_u64;
  ensures total <= 255000_u32;
} {
  let sum = 0_u32;
  for (
    i in 0_u64..count,
    invariant per_byte: sum <= 255_u32 * i
  ) {
    let w = deref(weights)[i];
    let wide = cvt::<u8, u32>(w);
    set sum = sum + wide;
  }
  return sum;
}
```

At each body entry the counted guard gives `i < count`, and the first
requirement gives `count <= len(weights)`, so the subscript is in range. The
header relation, the exact value image for `sum + wide`, and `wide <= 255`
prove the exact addition and the next header relation. At normal exhaustion the
checker substitutes `i := count`; the resulting `sum <= 255*count` plus
`count <= 1000` discharges the selected return's `ensures`. None of those
relations becomes a runtime check.

The first `for` header item must be the binding; every later item must be an
`invariant`. A zero-invariant counted loop still uses the closed header, which
renders on one line because it has no invariant to set apart:

```wf
for (i in 0_u64..count) {
  consume(i);
}
```

An ordinary loop without induction contracts remains `loop { ... }`. With
contracts it uses the same closed header, containing invariants only:

```wf
loop (
  invariant cursor_in_range: cursor <= limit
) {
  advance(cursor);
}
```

Labels occur after `for` or `loop` and before `(`. Header invariants cannot
carry `use` blocks. Their names do not exist before the loop, identify the
current arbitrary-iteration assumptions inside the body, and expire at body
exit. A counted loop may export the canonical relation produced by exact normal
exhaustion, with the binder replaced by the captured upper endpoint; `break`
does not receive that export. An ordinary loop has no exhaustion substitution.

## Induction semantics

A loop header is one simultaneous invariant batch:

1. on entry, every base goal is checked without assuming any member of that
   same batch;
2. only after the entire base batch succeeds are all members activated as the
   current iteration's body assumptions;
3. at every arbitrary reachable backedge, the checker constructs the whole
   next-header batch and checks it while the current batch remains available;
   no target may assume its own unproved next value; and
4. counted next-header goals substitute `i := i + 1`; ordinary-loop goals use
   the current backedge values.

If no backedge reaches the header, preservation is vacuous. `break`, `return`,
and propagation exits are not backedges. This is mathematical induction over
every reachable iteration, not a simulation of “the second iteration.”

A one-published-premise step such as `weigh` is automatic: after the exact
`set` image is substituted, AUTO subtracts `per_byte`, then DIRECT proves the
residual from the `u8` type interval of `wide`. A written use block there would
be redundant and invalid.

When a next-header relation really needs at least three published affine
premises outside the final fixed L0-image route, state it at the program point
where all ingredients exist:

```wf
invariant combined_limit: first + second + third <= first_limit + second_limit + third_limit {
  use first <= first_limit;
  use second <= second_limit;
  use third <= third_limit;
}
```

That local invariant is checked once. If it has the shape required at a
backedge, its published conclusion lets ordinary AUTO establish the header's
next relation. Diagnostics print that required source relation directly, so an
author can see what needs to be established before the edge.

## Explicit `use` certificates

A local invariant may have no block:

```wf
invariant ordered: lo <= hi;
```

or a finite explicit certificate:

```wf
invariant pair_bound: first + second <= first_limit + second_limit;
invariant scaled_bound: 3_u64 * first + 3_u64 * second <= 3_u64 * first_limit + 3_u64 * second_limit {
  use 3 * pair_bound;
}
```

Each `use` is either a named live invariant theorem or a relation that AUTO
proves under the current bindings. Every premise is checked against the same
snapshot entering the outer invariant. Earlier uses do not help later uses and
none of them publishes a fact. After the weighted combination is checked, only
the outer target is published.

A written factor is a proof integer, not a machine arithmetic operation. Factor
one must be omitted; factors begin at two. Repeating the same normalized
premise is invalid, regardless of spelling or factor. The checker derives and
checks the written combination in source order, and the target may be a direct
weakening of that result. After each premise's ordinary AUTO check, certificate
combination is linear in the written steps. The compiler does not search for a
different premise list, coefficient, case split, intermediate lemma, or
rewrite.

Names live in a proof-only lexical domain. A local name becomes available only
after its whole statement succeeds and remains available through its dominance
region. Live names cannot be shadowed. A named use resolves to the exact
declaration identity and immutable theorem, never merely to matching text.
Control-flow joins retain canonical equal facts, not source ordinals or proof
names. Writes change the current binding-to-value image; they do not make an
already proved theorem about an earlier immutable value false.

## One compiler proof context

This is a compiler, not a general proof service. The source AST, resolved
program, fact state, and checked program are ordinary internal compiler data.
An inconsistency among them is a compiler defect to fix in code and tests, not
an invitation to export a certificate, replay compiler-generated data, or add
a runtime self-check.

This cycle creates no `.wfproof` format, compiled-certificate cache,
incremental-proof protocol, or cross-module proof artifact. Such mechanisms may
matter to a future build system, but they cannot substitute for getting the
source checker and its semantics right, and they are not completion work here.

Every consumer submits its goal through the current semantic `ProofContext`:

```text
requires / branches / invariant / ensures
                  |
                  v
         current ProofContext
                  |
partial operation +-----> prove(context, exact goal)
                  |
                  +-- fixed equality and difference bounds
                  +-- fixed affine AUTO and written use steps
                  +-- interval / known-bit / congruence domains as specified
                  +-- ownership / initialization / typestate / effects
                  +-- layout / address / target-domain qualification
```

Here `prove(context, goal)` is the compiler's internal goal interface, not a
source keyword. The domains remain multiple small deterministic checkers behind
that interface, not one universal solver. An operation such as addition,
indexing, allocation, or a function call does not need to know whether its
evidence came from a guard, contract, loop invariant, or local certificate.

## Partial operations, target proof, and `par`

Every supported exact arithmetic operation, division/remainder, shift,
subscript, allocation fit, hidden counted-loop update, call requirement,
selected return, and system buffer range is proved before execution. Failure
to prove is a compile-time rejection; no hidden runtime branch is inserted.
Expected dynamic failure is represented as an ordinary typed result and handled
by real source control flow.

Source-domain proof does not replace target proof. Before emission the compiler
must still prove concrete layout, stride and byte ceilings, frame
materialization, address representability, target qualification, and every
selected operation's target-domain condition.

`par` consumes the same checked facts together with ownership, effects,
iteration identity, indexed-map or reduction relations, layout, target-domain,
and bounded queue/completion facts. It has no second proof language. Lack of an
optional `par` permission leaves the already accepted program sequential; it is
not a source rejection. If overlapping lowering is selected, every required
independence, index-disjointness, mapping, reduction, target, and bounded
completion premise is proved before emission. Proof syntax creates no runtime
branch, lock, dependency, scheduling marker, or task edge.

## Deferred boundary

Only external resource availability is deliberately outside this implementation
cycle: heap exhaustion, stack exhaustion, operating-system quota, and runtime
startup resources do not yet have a final Whitefoot source-level model. That is
a scoped temporary gap, not a change in the language's safety direction.

This deferral does not include allocation layout, address proof, frame layout,
target qualification, target-domain proof, parallel independence, or bounded
queue/completion protocols. A resource failure never creates a proof fact or
licenses an unproved operation.

## Completion evidence

This work is complete only when one exact revision has all of the following:

1. grammar, parser, resolver, checked model, semantics, diagnostics, erasure,
   and lowering agree on the source forms above;
2. compiler and compiler-independent conformance tests cover header induction,
   local certificates, contracts, control-flow joins, writes, partial-operation
   safety, target proof, and proof-driven `par` permission;
3. real programs exercise cross-function proof, loop proof, explicit guidance
   beyond AUTO's two-published-premise family, proof erasure,
   and sequential fallback when parallel permission is absent;
4. compile-cost and runtime measurements record the actual boundary without
   turning time into an acceptance rule;
5. all live documentation and derived syntax data match v0.40; and
6. canonical `make check` passes for the exact revision.

### Candidate measurement

Compile cost for this candidate is measured on the GitHub-hosted gate runners,
not locally: canonical `make check` ends with the wall time of each of its
stages, so the candidate's per-stage breakdown is compared there against the
same stages on the base branch. No local measurement is recorded, because only
runner timings are comparable across branches.

Proof erasure stays structural evidence rather than a timing inference: the
`source_proof_is_erased_before_typed_ir` test compares the full typed IR with
and without proof-only source, and separate compiler tests prove that a denied
optional `par` permission emits the sequential call sequence.

The activation those conditions gate is now recorded on this branch: v0.40 is
the ACTIVE identity, the outgoing v0.39 bytes are archived, and the derived
documentation and syntax data name v0.40. What remains is rules 2 and 3 — the
owner's approval of the exact revision and canonical `make check` on it.
