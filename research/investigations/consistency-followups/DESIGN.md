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
ordinary checking. Previously its prelude supplied the record the caller omitted:

```rust
check(&[], CompilerLimits::default());
check(&[SourceInput::new("empty.wf", b"")], CompilerLimits::default());
check(&[SourceInput::new("empty.wf", b"\n")], CompilerLimits::default());
```

The repair rejects the first input as an invocation failure before
prelude injection. PROG-2 supplies the requirement; TYPE-6 gives PRE-1 declarations
no writer source event. The second still reaches ordinary source checking and
fails FORM-2 for its missing final newline; the third is a valid declaration-free
source unit. No missing-entry rule or source outcome is replaced by this check.

The complete injected bundle still counts toward `max_sources`; the lower-level
`with_limits` transport still permits constructing an empty bundle whose parser
can diagnose the envelope. Both public projections, raw transport, the canonical
empty unit, and exact/insufficient prelude-plus-caller ceilings distinguish these
boundaries. Changing the transport's entire API or exempting prelude records from
resource accounting would change unrelated contracts without a supporting need.

## Assignment representations and resource records

The constructor audit found that `into_element_storage` sends every mutable Array
and Buffer assignment through Container/Storage before the old dispatch. Only a
constant Array can reach that dispatch; CONST-2 rejects it before construction.
No internal or test constructor supplies another route. The redundant checked
targets, their capture/kill/lowering consumers, and the exclusively produced
`StoreBuffer` IR instruction are removed. Live indexed reads remain.

Target capture before right-hand-side effects, constant-place diagnostics, state
invalidation and native assignment are the discriminating controls. Existing
coverage is retained and the target-before-RHS native case now also observes a
fixed Array and a boxed runtime Array. Focused native and semantic controls passed
after retirement; these cases remain in the full gate's ordinary test selection.

The generated POSIX heap-record writers previously aborted on EINTR while the
host stack-record writer retried it. A native baseline observation on `7127bcb6`
confirmed an empty record when the first write is interrupted, and only `{"res`
when interruption follows a five-byte write. Both stop with SIGABRT.

The emitted retry helper retains the supplied cursor and count on a negative
EINTR result. The existing writer advances on positive progress and stops on zero
or another error. Darwin and Linux use their selected ABI's errno accessor;
Windows remains on its existing diagnostic channel. The new native observer
checks exact remaining bytes and bounded call traces for interruption before and
after progress, zero with stale EINTR, and EIO. Both sequential and latched writer
forms passed all four schedules: interruptions complete the record; terminal
output failures still stop. Allocation refusal is injected into a four-element
allocation rather than obtained by exhausting host memory.

## Logical indices and target address operands

An empty element can have zero stride while a valid logical index exceeds the
target's signed address-index range. STOR-6 separately constrains indices and
scaled offsets actually used by emitted address computations. The alternatives
to compare are rejecting the logical index, qualifying the effective displacement,
or emitting a canonical zero operand when the selected layout has zero stride.
The selected candidate emits zero only for a zero-stride address step, determined
from the ordinary selected-target layout. Logical lengths, bounds and window
coordinates stay unchanged. A final empty field does not make a nonzero-stride
outer element step zero. This preserves the existing exact-address requirement
without rejecting header-only storage or depending on optimizer erasure.

STOR-6's target-derived length bound also needs a zero-stride definition: its
division applies only to positive strides; zero stride adds no count bound beyond
the source length type. Descriptor qualification and exact emitted-address
requirements remain. This is the v0.64 clarification, not a relaxation
of OP-4 or OP-9.

Zero/nonzero elements, nested layouts, construction/access, a narrow simulated
address domain and exact/insufficient descriptor ceilings distinguish the
alternatives. Inspect ordinary pre-optimization IR; there is no separate
facts-off compiler flag. Enormous full-array fills are emission-only. Native
coverage uses small zero-size values and a mixed nonzero/empty-field struct.

A subsequent Ring discriminator keeps the large capacity but inserts only two
zero-byte values, allocating only its header. At capacity `2^63 + 1`, WIN-1 and
OP-10 require two front placements to leave `head = 2^63 - 1`. Both the baseline
compiler and revision `3269302f` instead execute with `head = 0`: their emitted
`head + cap - 1` overflows on the second placement before conditional reduction.
The native source distinguishes that zero result from other unexpected values.

The repair computes `(head == 0 ? cap : head) - 1`. Placement already proves
`cap > 0`, and the Ring invariant supplies `head < cap`, so the selected base is
positive and no intermediate overflows. This changes neither source acceptance
nor the window representation, and uses the same result for the selected element
and published head. The existing zero-size boundary test now observes the two
placements and removals, the empty round trip, and capacity one under both
sequential and overlap lowering. Other address-only modular additions rely on
the positive-stride target bound; zero-stride address operands are zero, and
the head-advancing addition uses the separately safe offset one. Decoupling those
remaining calculations from these representation grounds is a deferred
improvement-validation task, not an established remaining observable defect.

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

The baseline distinguishes the obligations: empty/empty and empty/full Slots
both fail FN-8; separately formed disjoint ranges fail EFF-5, while overlapping
ranges fail its overlap check. An explicitly guarded selected-holder poststate
accepts; claiming that every possible origin changed fails INV-1. These results
confirm a precision limitation, not an unsound accepted program.

An isolated prototype substitutes every exact alternative of one joined holder
into an L0 query, proves every substitution in the same current L0 state, and
retains one relation about the selected holder. Both operands share the same
alternative; multiple joined holders, cover paths, ranges and mutable captured
indices are outside the candidate. A finite set of explicit alternatives avoids
cross-holder products but does not establish a bound in source size.

Independent research review found adoption blockers before integration:

- The existing `PlaceMap` supplies function-wide origins, including later
  rebindings, rather than a target snapshot at the query point.
- Equivalent Boolean and integer-domain proof consumers do not all take the
  new route. Passing an append precondition alone is insufficient.
- Registering candidate terms also registers measure siblings and standing
  bounds. Failed-query vocabulary changes can affect later consumers; that
  interaction needs explicit order and inertness evidence.
- Retained substituted cases need an authority establishing their completeness,
  not just checks that each listed case proves its own relation.
- Nested path choices can multiply explicit alternatives; the inherited origin
  resolver also has a depth cutoff. A source-polynomial, completion-defined
  family has not been established. Two substituted measure terms can request
  up to six measure terms per alternative, not the initial sketch's two.

The [isolated experimental revision](https://github.com/mbbill/Whitefoot/tree/ec59414abbe4897ffa33305d3e738cb58e3f1310)
retains the implementation, discriminating examples and the positive test that
still fails; none of that proof code enters this repair. Its 13 focused controls
produce 12 passes and one OP-2 normalization failure. Paired CLI probes confirm:

| Query | Baseline | Prototype |
| --- | --- | --- |
| Append through a choice of two empty Slots | FN-8 | Accept |
| The same earlier append with a later rebind to a full Slots | FN-8 | FN-8 |
| Separate empty/available requirements | FN-8 | Accept |
| Their `band` or `bnot(full)` composition | FN-8 | FN-8 |
| Nested positive joins with 2, 8 or 32 alternatives | FN-8 | Accept |
| Flat conditional-rebinding equivalents | FN-8 | FN-8 |
| A full alternative or a claim about all origins' poststate | Reject | Reject |

These invariance failures reject the candidate before performance selection.
No isolated checker-time scaling claim is made: successful CLI runs also lower
the program, unlike baseline rejections. The next bounded question is whether
whole-owned-root alternatives with one fixed query suffix can obtain point-current
authority and a bound in source roots. That narrower candidate is untested.

Do not add this family to the active specification on the strength of its simple
positive cases. Retain the bounded experiment and its failed discriminators as
research; reopen after defining current target authority and a source-size bound.
Branch-local range images also need presence and generation information: absence
of an alternative on a predecessor differs from losing a fact about an alternative
that is present. Blindly unioning endpoint images is not the proposed repair.

## Reserved names and declaration roles

OP-1's exhaustive reservation list excludes invariant declarations, but DIAG-1
previously includes them both in its payload-role inventory and its invariant
record paragraph. TYPE-6 supplies the separate proof lookup domain; it does not
own the contradictory reservation sentence.

The v0.64 rule admits every lexical IDENT in header and body invariant
declarations. `invariant cvt` and `use 3 times cvt` select the proof domain through
their grammar positions; `cvt::<u8, u64>(value)` still selects the operation.
Neither operation-versus-callable lookup nor field maximal munch supplies an
ambiguity here, so coupling proof names to the runtime operation inventory adds
restriction without the reservation's stated benefit. All three normative lists
are aligned and the obsolete FORM-3 invariant-role payload is removed.

The focused resolver matrix passed for `cvt` and all five mode words in header
and body declarations, checking exact named-premise resolution. Three conformance
cases cover execution with both spellings, a missing proof name, and an expired
header proof name. They use the ordinary conformance adapter; existing value/field
reservation and proof-scope controls remain.

The counted `for_binding` had a separate mismatch: OP-1 omitted it while
DIAG-1 and the resolver reserved it. The selected rule includes counted binders
in OP-1's runtime-name reservation, because a counted binder introduces an
ordinary value binding just as a `let` does. Permitting operation and mode words
only in counted bindings would make value-name availability depend on the
binding's syntax. This is a naming policy, not a claim that the loop grammar
cannot parse those names. The proof-only invariant domain remains separate.
The specification now lists `for_binding` explicitly; the resolver's existing
`for-binder` diagnostic mapping needs no behavior change. Conformance controls
reject `cvt` and `checked`, admit nearby names and a label named `@cvt`, and retain
the existing proof-name acceptance case. Resolver tests additionally cover all
five mode words with both labeled and unlabeled loops.
