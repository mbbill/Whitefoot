# Container lifecycle source probes

The current compiler executes five complete positive traces and rejects ten
nearby programs at the recorded boundaries. Eight rejections follow the
specified source/proof rules; the two boxed-block probes expose compiler
defects that are incorrectly reported as source errors. No explicit
unsupported-capability diagnostic, compiler crash, or incorrect accept was
observed. The important distinction is between missing language relationships
and compiler bugs: the dynamic capacity, result-field, and empty-linear-owner
limitations below are stated by the active specification.

Measured on 2026-09-06 at `0f537c77` (documentation-only successor to the
`eff095c7` integration baseline), with the gate-profile compiler, active v0.51,
Darwin arm64, and Apple Clang 21.0.0. These are source/lifecycle experiments,
not timing measurements or evidence for a different target.

## Reproduction and ownership

From this directory, after building the current compiler:

```sh
make check
make check WHITEFOOTC=/path/to/current/whitefootc
```

The parent experiment's `check` target owns this suite. `check` compiles and
runs every positive with `--no-overlap`; it checks exit 1, the exact source
rule and discriminating diagnostic detail, and absence of emitted LLVM for
every negative. Full diagnostics and `outcomes.tsv` are regenerated under the
ignored `build/` directory. `make clean` removes those outputs. The helper
files are ordered source-bundle inputs, not independent commands.

These expected outcomes characterize today's implementation of today's
specification. They are not conformance verdicts for a proposed language.
An intentional capability change must revisit the relevant expectation and
this explanation. Remove a probe when maintained language/compiler evidence
fully supersedes its design question; do not keep redundant parallel suites.

## Exact outcomes

| Fixture | Current outcome | Meaning |
|---|---|---|
| `pool_known_capacity` | Compiles, links, exits 0 | Explicit element replacement, direct measured helper result, struct field/destructuring, and local `Option` payload preserve capacity and empty length through this exact composition. |
| `pool_conservation` | Compiles, links, exits 0 | A two-block pool's take and return preserve the free-count relation through checked helper summaries, with no runtime capacity guard. |
| `pool_static_capacity` | Compiles, links, exits 0 | A nominal containing `FixedVector<u8, 4>` survives helper checkout, mutation, return, a second checkout/read, and final return; capacity 4 follows from the field type. |
| `pool_boundary_capacity` | `INV-1`, `UndischargedLocalInvariant` | A block known to have capacity 4 loses that fact after boundary insertion and removal; `cap_of(taken) >= 4` cannot be derived. |
| `pool_field_result` | `FN-9`, `InvalidPostconditionSelector` | The true identity helper cannot state a measure of its returned struct's field. |
| `pool_false_conservation` | `FN-9`, `Refuted` | A helper that removes one block cannot claim the free count is unchanged. This is an invalid contract, not a missing capability. |
| `pool_static_length` (generated) | `INV-1`, invariant `initialized` unproved | Changing the static-capacity invariant to initialized length 4 fails after the same checkout. Type-level capacity does not imply initialized length. |
| `pool_boxed_capacity` | `TYPE-7`, `deref requires a borrow holder` | Direct measurement of the checked-out box's run is incorrectly routed to the borrow-only dereference checker. |
| `pool_boxed_helper` | `FORM-8`, `RegionSpelling` | A user helper whose input is a run of boxes incorrectly requires an explicit store-region argument. |
| `linear_failure_cleanup` | Compiles, links, exits 0 | Two actual `linear Ticket` values are consumed on success; the one acquired before a later error is consumed on failure. Both executions run. |
| `linear_pop_empty` | `PROV-6`, `LinearValueNotConsumed`, binding `drained` | Both linear elements have been popped and consumed, and length zero proved; the empty run still cannot leave scope. |
| `linear_failure_run` | Same `PROV-6` detail at the error return | A partial construction is drained after acquisition failure, but its proved-empty run cannot leave scope. The diagnostic identifies the empty run, not the discharged ticket. |
| `linear_failure_leak` | `PROV-6`, `LinearValueNotConsumed`, binding `first` | Omitting discharge of the first ticket on the error edge is an actual linear leak and is rejected. |
| `ring_indexed` | Compiles, links, exits 0 | A four-slot ring containing logical bytes 2, 3, 4, 5 is consumed by logical indexing and produces checksum 14. |
| `ring_contiguous_view` | `BLK-0`, requirement `head_of(run) <= room_of(run)` unproved | The same actually wrapped storage is not one contiguous view. Rejecting this substitution is correct. |

## What the traces establish

### Pool relationships are partly compositional

The positive transfer trace uses three four-byte arena allocations so that
explicit replacement can first establish, then recover, a known element
measure. This is an experiment on placement rules, **not** a proposed pool
implementation or an acceptable extra-storage workaround. After the known
element leaves its explicit slot, the direct `Vector` helper states two
equalities (capacity and length). Ordinary field construction/destructuring
and a local `Option` match then keep both. The final capacity equality uses
two ordered local invariants, and emptiness uses one; all are discharged by
AUTO without `use` steps. A subsequent append and read really execute.

The pool conservation trace needs one take precondition, two take
postconditions, one return precondition, and one return postcondition.
Five local ordered invariants inspect the resulting count and space. No
dynamic recheck is required to return the leased block. This corrects the
overbroad claim that a library pool necessarily loses every useful fact.

However, [MSR-3 in the specification](../../../../spec/kernel-spec.md)
explicitly carries no element measures through `place_back`/`take_back`:
their implicit boundary index is not one of the admitted written place
offsets. `pool_boundary_capacity` exposes exactly that rule. It does not
silently add a capacity check or claim that more explicit affine steps can
recover an absent premise.

`pool_field_result` exposes a separate admission boundary. FN-9/CALL-4 admit
direct measured result ordinals and explicitly defer projected result
places. A true `cap_of(result.bytes) == cap_of(block.bytes)` relation is
therefore unavailable even for an identity helper. Fixing boundary-element
transport alone would not make a wrapped library container's contracts
compositional.

Architecture question: should invariant capacity/initialization facts travel
as type information, checked element/representation invariants, explicit
owning capabilities, or some combination? These probes establish the missing
relations; they do not show that another kernel pool nominal is necessary.

### Fixed capacity already has a type-level route

`pool_static_capacity` stores `StaticBlock { bytes: FixedVector<u8, 4> }`
inside `FixedVector<StaticBlock, 2>`. It initializes one block once, checks it
out through a helper, proves capacity at the field without a runtime guard,
increments every initialized byte, returns the block, checks it out again,
reads checksum 14, and returns it again. The count summaries are ordinary
checked helper contracts. There is no payload allocation or reinitialization
during checkout and return.

This is an actual positive discriminator: general element refinements are
not necessary just to retain this block's capacity. It does not retain the
fact that all four slots are initialized. The nearby `pool_static_length`
case is generated from the same source by replacing its capacity invariant
with `len_of(block.bytes) >= 4`; it fails at that exact invariant. The
positive processes the actual initialized prefix, using `len_of` as its
loop bound, rather than assuming a length-four result contract. That loop is
the program's processing semantics, not a branch added to assert a lost fact.

The representation tradeoff is explicit: these blocks are inline payloads
inside the free run and owned locals, rather than pointer descriptors to
independently stable blocks. Moving a block may move its payload and run
metadata. The constant, four-byte fixture is a correctness/proof test, not
evidence of cheap moves for realistic block sizes or stable addresses while
borrowed. No memory-layout or speed equivalence to the arena-backed pool is
claimed.

The alternative `Box<'s, FixedVector<u8, 4>>` probes cannot complete today:

- `pool_boxed_helper` reaches a user helper with the store region nested in
  the input run's boxed element. FORM-8 defines input region positions at any
  type depth and requires their inference. The compiler's
  `written_container_type_region` in
  [calls/user.rs](../../../../compiler/src/semantic/check/expressions/calls/user.rs)
  recognizes only a `FixedVector` whose element is `Vector`, omitting its
  boxed-element case, and wrongly reports that the region must be written.
- `pool_boxed_capacity` uses direct kernel checkout/return to isolate storage
  access from that helper defect. `cap_of(deref(block))` then reports that
  `deref` requires a borrow holder. TYPE-7 admits owned cell content access by
  `deref`, and MSR-1 admits measures of the resulting measured place. The
  borrow-only resolver in
  [borrows.rs](../../../../compiler/src/semantic/check/borrows.rs) is therefore
  the wrong path for this source, not a reason to reject boxed fixed storage
  as a language design. Later operations in this candidate remain untested.

Neither failed boxed attempt is repaired by destructuring the box and
allocating another one, by a fake initialization step, or by changing compiler
code. A future successful boxed probe must retain the single acquired cell
through checkout, use, and return, and measure its layout/cost separately.

### Empty ownership is distinct from element ownership

`Ticket` is explicitly linear; the positive error trace is not an affine
substitute. Its consumer destructures the ticket whole. The negative leak
omits exactly that discharge on the failure edge.

The two run traces perform all required element consumes and use one
`len_of(drained) <= 0` invariant to expose emptiness. PROV-6 still rejects
the scope exit because BLK-1 propagates element linearity to the run's type;
PROV-6 offers whole-value transfer or nominal destructuring, neither of
which completes this command for an empty run. This agrees with the
[existing design's recorded open problem](../../../investigations/containers-and-resources/CONTAINERS.md).

An empty-owner consume justified by a checked empty state, or an equivalent
representation-independent discharge rule, would address a real lifecycle
gap. Reclassifying the element as affine or returning an empty owner forever
would not complete the trace. These probes do not decide the final spelling.

### Wrapping is not itself a missing representation

The indexed ring passes through a helper whose one postcondition publishes
length 4. It remains one four-slot backing with physical head 1 and logical
contents 2, 3, 4, 5. The existing ring representation handles that sequence
correctly. Its logical subscript checks need no extra source invariants.

The negative asks for one contiguous view of all four initialized elements;
that request is false for this physical layout. A desired two-span consumer
would instead receive physical extents `[1, 4)` and `[0, 1)`. VIEW-2 currently
forms a whole non-wrapping view and provides no operation to construct those
two ranges from the wrapped owner. No invented candidate syntax is compiled
or reported as supported. A future two-span prototype must prove coverage,
order and non-overlap while retaining the one-backing resource contract.
Copying into a second backing would test a different cost contract.

## Limits relevant to architecture selection

The source fixtures contain no general sparse-slot store, cancellation API,
DMA interface, or asynchronous retirement policy. A pool's synchronous take
and return say nothing about whether the runtime retires staged iterations
out of source order. In any later staged probe, a borrowed backing must live
through join return, result consumption and the corresponding retirement;
DONE alone is not reuse permission.

The remaining architecture discriminator is a checked implementation that
combines these relationships: element facts survive pool checkout, aggregate
results publish them, and empty owners discharge after both complete and
partial construction. A separate two-span borrowing experiment would decide
whether the ring merely needs richer views or a different storage basis.
The inline fixed-capacity positive specifically removes one argument for
general element refinements. Initialized-length transport and the boxed
implementation gaps remain distinct questions. These results do not
demonstrate a need for general proof capabilities, nor do they establish that
a fixed set of compiler-owned state machines is enough.
