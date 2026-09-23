# Declaration and call boundaries after the reference redesign

This study compares the callable surface in the active v0.67 specification:
value-parameter and result `own`, mandatory result names, and named call and
construction operands. The question is which written distinctions still carry
information after x1 removed returned references and reference-valued types.
It does not reopen ownership transfer, reference access, expression nesting or
the source-format policy. The specification and implementation are unchanged.

## Requirements and comparison criterion

The selected direction is a signature-level comparison on complete functions,
contracts and calls. Before using the examples to select a form, require:

1. A reader can determine value, reference and range-reference parameter kinds,
   every result type and ordinal, effects and contracts from the declaration
   alone. No body, actual argument or later use selects the written form.
2. Copy, drop and must-consume behavior, explicit consuming uses, reference
   invalidation and the prohibition on returned references remain unchanged.
3. Proof-only result identities remain distinct from runtime locals, parameter
   entry images, reference exit state and routed Result payload identities.
   Generic instantiation and function-formal refinement preserve those roles.
4. A form has a deterministic grammar and one spelling selected by its grammar
   class. Unit and generic results must not require a spelling decision based
   on an inferred concrete type or on a checker's proof success.
5. Compare the exact errors labels currently detect: wrong, missing, duplicated
   and reordered labels. Also retain the counterexample in which two same-typed
   values are exchanged under otherwise correct labels; labels do not prove
   the writer's intended meaning.

Use scalar computation, generic ownership transfer, multiple results, routed
Result contracts, reference exit-state contracts and resource interfaces to
exercise these boundaries. The examples distinguish language rules; their
counts do not estimate real-world frequency or authoring productivity. A
shorter signature alone is not evidence of a better language. No runtime or
compilation speedup is claimed without a corresponding measurement.

## Current boundary and recovered history

The active [grammar and typing rules](../../../spec/kernel-spec.md) make these
three questions independent:

| Written element | Current semantic role | Relevant rules |
|---|---|---|
| `own` in `x: own T` | Selects the value-parameter alternative; the others are `&T` and `&[T]`. A reference kind cannot be supplied as `T`. | GRAM-2/3, TYPE-8, FN-1 |
| `own` in `r: own T` | The only admitted result mode; no reference result is legal. | GRAM-3, REF-3, FN-1 |
| `r` in a result binding | A proof-only name for one result ordinal; not a runtime slot, body local or part of callable-signature identity. | TYPE-5/6, FN-1/4/9, CALL-4 |
| Argument or field label | Checks the declaration's names and order at each use; does not choose an overload or permit reordering. | GRAM-8/10/11 |

The [v0.32 grammar](../../../spec/kernel-spec-v0.32.md) had three mode forms,
`own`, shared borrow and exclusive borrow, and returned borrows. It wrote an
unnamed `rtype` after `->`; FN-9 bound a result inside the postcondition form.
The [v0.33 change](https://github.com/mbbill/Whitefoot/commit/55a754340aa2be7f3374429534f16d197054aa36)
introduced a mandatory proof-only result binding while consolidating contracts.
The later result-list extension made those bindings identify result ordinals.

The [x1 spelling discussion](../access-effects/SPEC-AMENDMENT-MAP.md#surface-and-spelling)
recommended retaining result `own` because removing it would rewrite existing
signatures without a semantic gain. That is a historical recommendation, not
an independent argument for the present spelling: current
[decision practice](../../../docs/practice.md#evidence-guidance) excludes
internal migration cost as a language-selection ground before real adoption.
The resulting v0.60 grammar removed borrow results, leaving `rtype := "own"
type`; the current grammar retains that shape.

The [surface decision](../../../design/language/surface-form.md) keeps boundary
modes written on the ground that a reader cannot reconstruct them there.
Revisit that ground separately for each element: explicit `&` already
distinguishes both reference parameter forms from a value type, and a result
has no remaining mode alternative. Type, effect and contract information still
has to be stated. This observation does not select a result-naming mechanism.

The [construction decision](../../../design/language/surface-form/construction-form.md)
instead relies on rejecting label/order mistakes between same-typed operands.
Changing ownership syntax does not invalidate that independent reason.

## Candidate parameter and result modes

Compare the current form with making a written value type itself denote a
value parameter or owned result:

```text
Current:   fn mix(x: own u32, y: own u32) -> result: own u32 pure
Candidate: fn mix(x: u32, y: u32) -> result: u32 pure

Current:   fn inspect<T>(value: &T, range: &[T]) -> result: own u64 reads(value), reads(range)
Candidate: fn inspect<T>(value: &T, range: &[T]) -> result: u64 reads(value), reads(range)
```

This is a mandatory spelling replacement for the whole grammar class, not an
optional abbreviation and not a change to `move`. The semantic distinction
between value and reference parameters remains visible without interpreting a
body or substituting a generic argument. Result naming is held constant in
these two comparisons so it can be assessed on its own grounds.
