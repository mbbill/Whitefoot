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

## Results

Run on 2026-09-15, Darwin arm64, `rustc 1.98.1 (48a229cea 2026-09-01)`.
The prior criteria were committed as `a6f9051f` before `model.rs` existed.
Reproduce with `make -C research/experiments/access-state check` from the
repository root. The root `make check` invokes the same target. Both binaries
use `rustc --edition=2024 -D warnings -C opt-level=2`; the test binary also
uses `--test`. Source formatting and `forbid(unsafe_code)` are checked.

| Observation | Result |
|---|---|
| Focused semantic tests | 13 passed. |
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

The collection/family and full-language rules are proposals in the linked
design, not results of this experiment. The two alternative language systems
are compared by their stated rules, not by running competing compilers.
Bounded success does not settle full soundness, inference ergonomics, or
practical checking and generated-code costs. A counterexample outside the
fragment would reopen the corresponding proposed rule.
