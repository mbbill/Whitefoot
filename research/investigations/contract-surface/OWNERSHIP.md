# Ownership transfer and reference-access forms

This investigation compares the remaining ownership surface after signature
`own` was removed. Its baseline is specification v0.69 and repository revision
`0f22b026b`. The concrete consumers are the maintained HashMap and Deque
libraries and the owned-link cursor program. The active specification remains
the language authority; candidate spellings below are not accepted syntax.

## Requirements and comparison criterion

Record these criteria before trying the alternatives:

- Preserve copy/drop capabilities, exactly-once consumption of noncopy owners,
  whole-owner partial moves, linear-value obligations and derived cleanup.
- Preserve reference-holder identity, rebinding, captured selectors,
  invalidation, overlap and declared effects. A shorter access must select the
  same path and establish the same partial-operation obligations.
- Determine a written use from its syntax, the already known local kinds and
  generic bounds. Do not select copying, moving or borrowing from later uses,
  from the selected concrete generic instance, or from a preferred overload.
- Compare the same operation and result contract, including error and
  replacement outcomes. Do not label an API obsolete merely because it
  consumes an input and produces an output.
- Distinguish lexical shortening from fewer ownership transfers or runtime
  operations. Source counts in these examples do not establish population
  frequency, writer productivity or performance improvement.

The alternatives are discriminated by paired normal/error examples and by
counterexamples: copy and noncopy generic instances, owner use after transfer,
stored linear fields, reference aliases versus referent reads, holder rebinding
versus referent assignment, ancestor replacement, and effect overlap.
An alternative that only shifts the same obligation into another mandatory
declaration has not removed that obligation.

## Questions

1. Do current container interfaces still require avoidable owner-in/owner-out
   mutation? Compare ordinary writes through a reference with a consuming
   rebuild, release and atomic owned transformation.
2. Should ordinary affine expressions, match scrutinees, propagation and
   destructuring share one consumption spelling? Compare the current rules,
   explicit markers at every consuming boundary, and type-directed consumption
   of bare value places. Keep last-use inference a separate alternative.
3. Can reference projections become shorter without implicit value reads or
   confusing holder assignment with referent assignment? Compare the current
   `deref` step, an explicit projection spelling, and automatic reference
   projection. Whole-referent access and forwarding a reference must remain
   distinct observations.

## Current rule boundary

OWN-1 requires `move` for ordinary affine place expressions and rejects it on
copy values. FN-2 checks that spelling against a generic body's bounds once;
an unbounded generic `move` may denote copying at a copy instance. OWN-13 and
ERR-3 independently supply consuming contexts for an own-place match and a
bare affine Result propagation operand. Thus written `move` is neither every
ownership transfer nor an unconditional promise of noncopy behavior.

REF-1 makes a bare reference binding an alias or a forwarded reference.
TYPE-7 requires `deref` to reach its referent. SET-1 distinguishes rebinding a
reference holder from storing through it. `Box.inner` is a separate ordinary
owned-field step. Removing a repeated `deref` spelling must not erase those
distinctions.

## Consumer comparison

The comparison and checked evidence will be developed here. This document is
the existing ownership-surface investigation's home; it will be superseded in
place if a later comparison replaces its conclusions. Maintained regression
cases belong in the formal test system, not in this research directory.
