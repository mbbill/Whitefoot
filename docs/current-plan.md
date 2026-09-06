# Current Plan: finish source-carried proof in the compiler

Status: IMPLEMENTED AND ACTIVATED as v0.40 on
`codex/source-proof`.

The active language authority is the specification at `spec/kernel-spec.md`;
its version and digest are the chain tail in `governance/APPROVALS.md`. On top
of the v0.40 proof surface this plan delivered, v0.41 added the comparison
symbols and the call-site `::` delimiter, v0.42 the canonical region spelling,
v0.43 the loop-body region block and the associative join, v0.44 the fact
machinery ([MSR-3], [MSR-5], [CALL-4], [CALL-6]), v0.45 the interval an
admitted product already proved ([ENT-3.S14]), v0.46 the clause relation
and the measure atom that discharges it, v0.47 the named const as an affine
atom, v0.48 the `use` premise and its named multiplicity, and v0.49 the fold
that names a declaration rather than its expansion. Each superseded version is
archived at `spec/kernel-spec-vN.md` with its merge-time record in
`governance/APPROVALS.md`. Nothing merges to `main` until the owner approves
the exact revision and canonical `make check` passes on that revision. This
document records technical direction and sequencing; it grants no permission
and adds no workflow gate.

v0.43 carries two independent amendments. The first makes every `loop_stmt` and
`for_stmt` body a region block [OWN-3, OWN-11]: the body introduces one unnamed
region over exactly that body, a borrow written directly in the body takes it
and is written bare, and a `region_stmt` that is the body's only statement is a
second spelling of that one region and a hard error citing [FORM-8]. A block the
body writes another statement beside is strictly narrower, is what [OWN-6]'s
statement-scope judgment needs, and stays legal. The second repairs [ENT-6]'s
control-flow join, which was not associative: a delta atom an earlier join
minted counted as an ordinary nonconstant term at the next one, so a three-way
demux was accepted written as a flat `match` and refused written as nested
`if`/`else`. Each input image is now normalized by folding earlier delta atoms
back into the constant interval they stand for, so acceptance no longer depends
on the shape of the join.

v0.44 adds four rules and retires none (136 remain). [MSR-5] lets a `requires`
or `ensures` operand be a measure of a place, so `ensures len(rest) >= len(out);`
is a written clause where it was a parse rejection, and the define-per-measure
spelling is gone. [MSR-3] gives every contract operand one denotation keyed on
its parameter's mode: an `own` operand read at a caller denotes the call datum,
the value at transfer, which no consume and no later write kills; a `&uniq`
parameter's measure is inadmissible in a source-declared `ensures`, because the
callee cannot name the caller's object at a point after its own writes.
[CALL-4] states the contract vocabulary over the one result a declaration has
and records the deferred widenings. [CALL-6] states once how a declared
relation is instantiated at the call, established on the normal continuation,
and restricted to its routed arm, and refuses at the declaration a contract
whose published relations contradict each other. The batch that carried it is
B1 of the container and resource design under
`research/investigations/containers-and-resources/`.

v0.45 adds and retires no rule (136 remain) and amends [ENT-6] and [ENT-3] in
place. [ENT-6]'s fixed interval-product rule proves an inclusive interval for
each operand of a non-constant multiplication and forms the four products of
their endpoint pairs; the multiplication is admitted exactly when all four lie
in the result type, and the rule then discarded them. [ENT-3]'s new source S14
establishes the least and greatest of those same four products on the value
the multiplication binds, so an admitted product no longer produces a value
with no bound and the operation that follows it has the premise the checker
had already proved. Both published relations are constant bounds against the
distinguished zero term, so their [ENT-5] support is the bound value alone: a
later write to an operand leaves them true, a write to the bound place kills
them, and no relation over the operands, new term, or automatic premise route
is added. A written `use` remains the only way a product participates in a
certificate, and a domain discharged by the finite L0 or affine-clause route
publishes nothing. The evidence that selected it is in
`research/investigations/binary-arithmetic/`.

v0.46 adds and retires no rule (136 remain) and amends [FN-8], [ENT-3] and
[ENT-6] in place. Three sentences move together because none is useful alone.
[FN-8] admits exact addition, subtraction and multiplication in a clause and
reads them over the mathematical integers, the carve-out [INV-1] already gives
an `affine_expr`; a clause is erased before lowering and evaluates nothing, so
a row total over the mathematical integers states a relation where it would
otherwise request an operation. Division, remainder, negation, absolute value
and the shifts stay inadmissible. [ENT-3] makes a measure term an affine atom,
one per measured place, identified by its root binding and tightened by the
L0-to-affine index, which is the admission v0.44 recorded as deferred. And
[ENT-6]'s affine route discharges a comparison goal whose normalization is
affine whether or not it also projects to L0, the projection being what the
evidence names rather than what the route requires. Together they make
`requires len(out) >= 2 * len(src)` — the precondition of every expansion
codec — writable and dischargeable; each alone leaves it refused. The evidence
is in `research/investigations/binary-arithmetic/`.

v0.47 adds and retires no rule (136 remain) and amends [INV-1] in place. An
integer-typed named const is an affine atom, folded at formation to the one
closed value it declares. It was already an [ENT-2] constant term, so the
exclusion made one declared value mean a number everywhere except in the
relations written about it: a limit declared once had its digits rewritten
inline in every invariant and every `use` that named it, and a stale digit is
a silent divergence between what the code enforces and what the proof states.
Folding at formation keeps the admission free of consequence — no atom kind,
image, kill, or join changes, and the same relation over a const and over its
literal is byte-identical, including in a failure's rendered residual. A
const-generic parameter is symbolic rather than closed and is not this
admission.

v0.48 adds and retires no rule (136 remain) and amends [GRAM-4] and [PRF-1] in
place. A `proof_use` cites exactly one premise — the new `use_premise`
production, an invariant name or a delimited relation — and states its
multiplicity as `N times` before it. The multiplicity was spelled with `*`,
which claimed it was a multiplication whose right operand is a relation; the
form read as `n * bool`, a term multiplicity was undecidable in strong-LL(2)
because after `use IDENT *` the separating token is arbitrarily far away, and
the [FORM-2] stated space before `(` existed to carry a distinction the parser
could not see. Naming it removes all three, and the whitespace rule becomes the
ordinary keyword-paren space a `for_stmt` header already states. The
multiplicity may now name an unsigned integer value: the accumulated sum is
then a polynomial of degree at most two, every nonlinear monomial must fold to
the value image an admitted exact multiplication already bound, and a sum that
keeps one rejects. Nothing else in the language carries a nonlinear term. The
capability is a matrix multiply's inner index at a runtime stride, whose
certificate is one term-scaled premise and one plain one; the evidence is in
`research/investigations/binary-arithmetic/`, with the design in its
`PROOF-SURFACE.md`.

v0.49 adds and retires no rule (136 remain) and amends [PRF-1] in place. A
multiplication's operand and a written multiplicity name the same value when
they name the same declaration, not when their images coincide. v0.48 folded by
the images, and a local's image is transparent — `let stride = width +
padding;` gives `stride` the image `width + padding` — so a product over
`stride` and a certificate scaling by `stride` arrived at the fold as different
arithmetic and nothing matched. Measured on one shape with only the derivation
varying, a stride copied from a parameter accepted while `width + padding`,
`width + 4`, and `2 * width` all rejected; a stride is definitionally derived,
so the feature reached matrix multiply, where the stride happens to be a
parameter, and nothing else in its own domain. Each such binding now
contributes one opaque handle that both sides name. The handle exists between
the fold and the residual and is replaced by the image it stands for before
anything is proved, so every other premise, the target, and the residual read
exactly what they read before — which is why no snapshot or conformance verdict
moves. Publishing the handle's defining equality as a fact and replacing the
binding's image with it were both built first: the first is invisible to the
residual, which is the direct L0 route by rule, and the second makes every
ordinary premise about the binding need that equality to prove. The evidence is
in `research/investigations/binary-arithmetic/`.

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
fn weigh(weights: &buffer<u8>, count: own u64) -> total: own u32
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
  use (first <= first_limit);
  use (second <= second_limit);
  use (third <= third_limit);
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
  use 3 times pair_bound;
}
```

Each `use` is either a named live invariant theorem or a relation that AUTO
proves under the current bindings. Every premise is checked against the same
snapshot entering the outer invariant. Earlier uses do not help later uses and
none of them publishes a fact. After the weighted combination is checked, only
the outer target is published.

A written bare-decimal multiplicity is a proof integer, not a machine
arithmetic operation. Multiplicity one must be omitted; written decimals begin
at two. A named multiplicity reads a live own unsigned integer value in the
same entering snapshot, which is what makes scaling sound without a further
obligation, and it makes the sum a degree-two polynomial that must fold back to
affine against admitted exact products before the residual forms. Repeating the
same normalized premise is invalid, regardless of spelling or multiplicity. The checker derives and
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
