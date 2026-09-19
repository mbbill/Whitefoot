# Access-state checking experiment

## Question and prior criteria

Recorded before implementing the model, 2026-09-15. The consumer is the
[ownership-system investigation](../../investigations/access-effects/RESEARCH.md).
Keep this executable evidence while that investigation relies on it; replace
or remove the model when a stronger experiment supersedes these observations.

This experiment tests a finite first-order fragment, not WF source programs.
Its common objects have data and a reference field. The same operations must
cover retained graph edges, retargeting, shared control state, partial
initialization, ownership transfer, and individual reclamation with physical
slot reuse. No resource or function name may select a checking rule.

Discriminating criteria, fixed before results:

1. A reference's target is stable information; current live/initialized state
   is separate. Multiple aliases allow sequential writes. Reclaiming a target
   removes access through every alias, including aliases stored in objects.
2. An owner moves without renaming its backing allocation. Retargeting a
   reference field does not retarget an already loaded alias. A fresh logical
   allocation can use a freed physical slot without reviving old locators.
3. Generic bodies admit possibly equal reference arguments. State changes
   conservatively affect possible aliases; callers substitute relationships
   into checked signatures. They do not inspect or clone the body.
4. Effects on separate payloads can be independent despite a common stored
   link. Effects through that common link conflict. Partial initialization
   blocks a nested call that would read the affected field.
5. For exhaustive bounded instruction sequences, every accepted sequence must
   execute without stale or uninitialized access in an independent physical
   storage oracle. An address-only execution must have the same observed
   values. Deliberately faulty checkers must expose counterexamples, showing
   that the oracle can discriminate the intended failures.
6. Report counts and bounds rather than treating bounded enumeration as a
   soundness proof. Measure checker operation counts for increasing symbolic
   parameter sets; do not infer a bound for a complete language from this
   fragment. No runtime-cost claim follows from interpreter timings.

The oracle may carry allocation generations to detect errors. Those are test
instrumentation, not a proposed runtime mechanism. The address-only machine
has allocator occupancy bookkeeping but no generation or borrow check.
Neither interpreter establishes native-pointer lowering or LLVM validity.

## Local baseline criteria

Committed as `4eb8452c` before implementing this increment. It uses the existing checker and
physical interpreter, restricted to the
[local straight-line baseline](../../investigations/access-effects/DESIGN.md#local-straight-line-baseline).
Two independent initialized scalar objects and their fixed owners are created
at entry. Two local locator slots can be introduced and rebound. The body has
only locator creation/rebinding, read, store, take, and release; it contains no
branch, loop, call, reference field, owner move, or further allocation.

The prior experiment could not express rebinding an existing local locator:
`Alias` only introduces an unoccupied binding. The criterion for the increment
is a general locator-assignment rule that preserves the identity captured by a
prior copy and does not permit an owner to be overwritten. All previous tests
remain in place.

Discriminating criteria for this increment:

1. Human-readable examples specify accepted observations or the first illegal
   operation before execution. The six motivating cases are copy/rebind,
   sequential alias writes, read after take, restoration through an alias,
   access after release, and repeated owner consumption. Include non-owner
   disposal and unconsumed exit obligations as boundaries.
2. For every reachable state in the finite two-object/two-locator resource
   domain, compare both admission and resulting resource state for every
   instruction in the fixed alphabet. Explore to closure, not to an instruction
   depth. Rejecting an oracle-valid instruction is a precision failure here;
   accepting an oracle-invalid instruction is a safety/accounting failure.
3. The oracle uses the physical interpreter plus explicit test-only ownership
   and binding checks. It must not consult the checker's targets, liveness,
   initialization facts, or verdict. Payload values do not select access or
   control, and the compared state abstracts them to initialized/empty. This
   criterion concerns resource relations, not arbitrary scalar-value properties.
4. Check exit obligations at every reachable resource state. Do not silently
   treat live owners as a completed program. Invalid transitions must be
   compared before being excluded from further exploration.
5. Deliberately corrupt a retained alias, initialization fact, and ownership
   record and demonstrate that the comparison detects each discrepancy.
6. Report the state/transition counts and limitations after execution. This
   closed finite model is not a proof for arbitrary object counts, branches,
   runtime layouts, hidden allocations, or a native backend.

## Local baseline results

Run on 2026-09-15 with the toolchain below. Reproduce this scope, including a
statement-by-statement state trace, with:

```sh
make -C research/experiments/access-state local
```

The new [local test module](local.rs) exercises the ordinary checker through
`Rebind` and explicit end-of-procedure ownership checking. The physical
interpreter follows addresses; a separate test-only ledger records the owners
and rejects invalid binding/disposal operations that the interpreter alone
trusts. It does not consult the static checker. The state comparison records
each binding's target and ownership role and each object's live/initialized
state. Dead objects have no relevant initialization state.

| Observation | Result |
|---|---|
| Named examples | 16 matched their specified values, first rejected operation, or exit error. |
| Reachable resource states | 81, explored to closure with no instruction-depth bound. |
| Fixed instruction alphabet | 40 instructions: read/store/take/free over four bindings, rebinding over all binding pairs, and introducing either locator from any binding. |
| Admission comparison | 900 admitted transitions and 2,340 rejected transitions; all 3,240 matched the oracle. |
| State and exit comparison | Every transition preserved exact resource-state agreement, including failure states; all 81 exit judgments agreed, with 9 states having no outstanding owners. |
| Corrupted-state controls | Wrong saved target, falsely restored initialization, and a resurrected owner record were each detected. These mutate facts, not complete alternative checker implementations. |

Each object has three relevant states (initialized, empty, dead); each locator
is absent or targets either object. Entry owners are fixed and consumed only
by release, so their presence is determined by object liveness. The reachable
resource domain has `3^2 * 3^2 = 81` states. Scalar payloads, accumulated output,
effect history and query counters cannot affect admission or target selection
in this restricted instruction language; they are not part of the exploration
key. The exploration checks every edge of this resource-state quotient, not
every concrete integer payload or every possible language feature.

These excerpts explain the executable examples; they are not WF grammar.
`a` owns A containing 10, `b` owns B containing 20, and `p = ref(a); q = p`
initially makes both locators designate A. Successful complete examples also
explicitly release both owners, unless already consumed.

```text
p = ref(b)
put(p, 9)
read(q)                  // 10: q still designates A
read(p)                  // 9: p now designates B
```

```text
take(p)
read(q)                  // rejected: A is empty
```

```text
take(p)
put(q, 7)
read(p)                  // 7: the same live slot is initialized again
```

```text
release(a)
put(q, 7)                // rejected: writing cannot revive A
```

Releasing through `p` is rejected for lacking ownership; releasing `a` twice
is rejected at its second use because the binding was consumed. Rebinding an
owner is rejected instead of losing its obligation. Exiting after releasing
only A is rejected because B still has an owner. Taking twice without a put is
rejected at the second take. Copying an inert locator is allowed without
dereferencing it, consistent with candidate A's stated locator semantics.

The result supports exact propagation for this closed local domain. It does
not establish that target sets or relational joins are adequate: this probe
contains no source branch. The exploration's own branching enumerates possible
test programs. Owner movement, storage relocation/reuse, function boundaries,
fields, loops, and arbitrary object counts remain outside this result.

## Results

Run on 2026-09-15, Darwin arm64, `rustc 1.98.1 (48a229cea 2026-09-01)`.
The original criteria were committed as `a6f9051f` before `model.rs` existed.
Reproduce with `make -C research/experiments/access-state check` from the
repository root. The root `make check` invokes the same target. Both binaries
use `rustc --edition=2024 -D warnings -C opt-level=2`; the test binary also
uses `--test`. Source formatting and `forbid(unsafe_code)` are checked.

| Observation | Result |
|---|---|
| Focused semantic tests | 17 passed: the original 13 plus four local-baseline tests, including the 16-example table. |
| Three payloads plus shared control state, middle release and replacement | Allocation slots `[0, 1, 2, 3, 2]`; observed payload values `[11, 33, 44]`; the program also executes with exactly four physical slots. |
| Bounded enumeration | 646,421 attempted instruction extensions; 560,265 accepted prefixes; 2,608 accepted allocations reuse a physical slot. No accepted prefix fails the oracle or differs in observed values from address-only execution. |
| Fault: reuse a dead logical identity | Counterexample found: allocate, retain alias, free, reallocate, then write through the old alias. The oracle rejects the stale access while the address-only machine can access the replacement. |
| Fault: re-evaluate a saved field path | The checker accepts a saved locator after its original target is freed because it wrongly retargets it. The physical oracle rejects the access. |
| Fault: distinct formal names imply separation | A generic `take(y); read(x)` body is wrongly accepted; the caller can pass the same target twice, and the oracle detects the uninitialized read. |

The enumeration uses at most six outer instructions and three named registers.
The instruction alphabet is allocation, alias copy, owner move, data read,
write, take, link store/load, free, and a call to a checked two-parameter writer.
A new destination is the first unused register; literals are the current depth
or the fixed store value. Function calls can execute multiple body instructions.
The physical capacity equals the outer depth, so these traces cannot exhaust
it. These are safety checks on prefixes, not counts of complete WF programs or
of all programs up to six machine instructions. Accepted prefixes can still
have live owners; scope cleanup and final linear consumption are not enumerated.

The focused cases additionally cover cyclic links, stored inert locators,
saved links across a retargeting call, a moved owner preserving backing
identity, same-target and disjoint-target calls using one body, signature
effect completeness, shared-control effect projection, and rejection of a
nested read while an aliased field is uninitialized. Two different checked
bodies with the same state/effect signature preserve caller acceptance and
each retain their own source-order values.

The oracle is independent of the checker's symbolic target environment. It
uses a first-fit physical store, pointer values carrying the allocation event,
and runtime checks for stale or uninitialized access. A second instantiation
of the interpreter uses address-only pointers and a zero-sized stamp, so it
cannot detect generation mismatch. They share interpreter control logic;
agreement is not independent validation of that interpreter. Neither is a
native backend or evidence that all lowering checks can be erased.

## Checking-work measurement

For one generic body writing N possibly aliased scalar arguments, verification
performs the following numbers of may-alias queries:

| Parameters | Queries |
|---:|---:|
| 8 | 56 |
| 32 | 992 |
| 128 | 16,256 |
| 512 | 261,632 |

These equal N(N-1): each write compares its target with every other symbolic
target, without enumerating equality partitions. Set lookup also has its own
cost. This is a deterministic operation count for this model, not a measured
compile-time, memory-use or whole-language complexity bound. No source or
backend performance comparison is claimed.

## What this does and does not select

The observations support access/state checking as a concrete core worth
developing: it admits useful aliasing while rejecting the tested stale,
uninitialized and unaccounted disposal operations. Shared links do not force
payload effects to merge, and signatures can export those links and changes
without caller access to the body. Physical slot reuse does not require
logical identity reuse or runtime generation checks in the address-only model.

The prototype operates on initialized scalar-data cells with one optional
reference field. It has fixed-root allocation, finite straight-line bodies,
direct calls, and a deliberately small signature language. Function result
packages, general branching and recursive loops, dynamic family predicates,
variable-sized range layouts, affine fields, arbitrary higher-order captures,
native pointer provenance and parallel execution are not implemented here.
The independence tests check footprint judgments, not concurrent executions.
Calls in the generic body verifier are not modeled; the nested-read witness
is a direct signature-checked call in a partially initialized caller context.

Allocation and physical occupancy management are interpreter primitives. The
shared control cell is an ordinary reference-field witness; it is not a
verified implementation of that first-fit allocator. Allocation effects,
capacity contracts and an allocator representation proof are not checked by
this prototype. In particular, no allocation-overlap permission is inferred
from the model's fresh-name creation step.

The collection/family and full-language rules are proposals in the linked
design, not results of this experiment. The two alternative language systems
are compared by their stated rules, not by running competing compilers.
Bounded success does not settle full soundness, inference ergonomics, or
practical checking and generated-code costs. A counterexample outside the
fragment would reopen the corresponding proposed rule.
