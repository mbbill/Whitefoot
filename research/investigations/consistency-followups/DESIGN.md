# Post-redesign audit follow-ups

This investigation develops the unresolved API, representation, target and proof
questions retained after PR #84. Its initial source baseline is main at
`7127bcb6`. The active specification owns acceptance; examples and implementation
behavior below are evidence, not additional rules. Keep this account with its
eventual results and rejected alternatives rather than creating a parallel work
queue; outstanding work remains in `docs/todo.md`.

## Source input and compiler-owned prelude records

PROG-2 requires a nonempty sequence of logical source records and classifies an
empty sequence as an input-envelope failure. The public driver constructs a
`SourceBundle::with_prelude`, which adds compiler-owned declaration records before
ordinary checking. Consequently the following inputs currently differ from the
CLI's requirement for at least one source path:

```rust
check(&[], CompilerLimits::default());
check(&[SourceInput::new("empty.wf", b"")], CompilerLimits::default());
```

The distinction to settle is absence of a caller-provided source record versus a
present record containing no declarations. Prelude availability, caller record
identity, and the resource ceiling for the complete internal bundle are separate
properties. A repair must distinguish the two inputs without turning resource
exhaustion into source rejection or silently changing how injected records count
toward existing limits. The discriminating controls are both public projections,
raw bundle construction, a present empty record, and limits at the prelude-plus-
caller boundary. The resulting policy and its ground will be recorded here.

## Empty release actions and assignment representations

The existing checked-place design uses Container/Storage paths for mutable array
and buffer element assignment. The remaining flat set-target representations
must be judged by their constructors and complete consumer inventory, not by
their names. Retirement is justified only if no accepted source or necessary
internal consumer constructs them. Target capture before right-hand-side
effects, constant-place diagnostics, state invalidation and ordinary native
assignment are the discriminating behaviors; unrelated read representations
are outside that equivalence claim.

The generated POSIX heap-record writers and the host stack-record writer differ
on interrupted writes. An EINTR repair must retain the output cursor on an
interrupted negative result, advance it on a positive short write, and terminate
on zero or a different error. A native observer will inject those outcomes and
check both the exact record and a bounded call trace without exhausting host
memory. Windows behavior is a separate platform path.

## Logical indices and target address operands

An empty element can have zero stride while a valid logical index exceeds the
target's signed address-index range. STOR-6 separately constrains indices and
scaled offsets actually used by emitted address computations. The alternatives
to compare are rejecting the logical index, qualifying the effective displacement,
or emitting a canonical zero operand when the selected layout has zero stride.
Zero and nonzero elements, nested layouts, construction and access, target width,
and ordinary facts-off emission distinguish these alternatives. The large logical
count is a checking/emission witness, not a native fill-loop workload.

## Reference joins and bounded proof precision

Joining references to two newly empty Slots loses a useful target-relative
length fact under the current fixed proof routes. Ranges formed in separate
branches similarly lose branch-local endpoint images. A useful extension must
accept the empty/empty and disjoint/disjoint controls while rejecting empty/full,
overlapping, stale-capture and rebound-holder controls. A write through the
selected holder may establish its new state, never the new state of every
possible origin. Any candidate must state a finite source-derived fact domain
and a checking-cost criterion before measurements; path enumeration or a second
unbounded relation solver is not an assumed solution.

## Reserved names and declaration roles

The operation-reservation role list and the invariant-name requirements need a
single consistent interpretation. Current resolver behavior is not sufficient
ground for choosing that interpretation. Compare operation calls, ordinary
value/field bindings, certificate names and their `use` references, including
header and local invariants. The recommendation must identify the grammar or
lookup ambiguity it prevents, or explain why the contexts already distinguish
the names, and update every normative role list and derived diagnostic together.
