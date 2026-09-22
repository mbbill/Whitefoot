# Experimental joined-reference L0 query projection

This branch is an isolated research prototype based on `7127bcb6`. It is **not an
admitted specification family** and must not be merged as compiler conformance
work. The active specification is deliberately unchanged. Its purpose is to test
whether a small query-only rule can recover useful facts for a reference joining
two independently constructed empty Slots without introducing path-history search.
Keep this note with the experimental revision; supersede it if a later experiment
replaces the candidate. The maintained compiler and specification remain the
baseline, and their current rejection of these cases is not asserted to be a bug.

The candidate substitutes every exact target of one joined holder simultaneously
into an L0 relation, asks the existing closed L0 state for every result, and
concludes only the selected-holder relation. It creates no origin equality and
publishes no new source facts. A distinct `SelectedTargetRelation` node records
case relations and proof parents. The prototype does not change range-image joins.

The pre-experiment criterion required positive empty-Slots precision, corresponding
full-origin refusal, selected identity and poststate, stale/loop controls, and a
source-polynomial deterministic domain. No budget or timer selects acceptance.

The focused `selected_targets` harness has **12 passing controls and one retained
expected-positive failure**: with both possible Slots holding one value, a selected
subscript is admitted but `4_u64 / deref(selected).len` rejects at OP-2. Do not
weaken this control to declare the prototype complete. Both ordinary construction
facts and same-holder field-relation queries demonstrate useful local precision;
all-origin poststate claims and mixed empty/full choices remain rejected.

Paired source probes also expose failed adoption criteria:

- Adding a later `set selected = &full;` changes an earlier fresh-empty append
  from accepted to FN-8 rejection, because target authority is function-global.
- Separate requirement leaves are admitted while their `band`/`bnot` composition
  is not. Integer-domain alternate normalization has the related limitation above.
- Nested positive selections of 2, 8 and 32 empty targets are admitted; corresponding
  flat conditional-rebinding positives are not. Their negative controls reject.
- The retained ledger checks substitutions and parents but does not independently
  establish that every structural target is represented by a case.
- A bound in explicit alternative count is not a demonstrated bound in source
  size. Descendant choices can multiply paths; the reused resolver also has a
  depth-limited unresolved case. Failed queries register terms/standing rows even
  though they publish no relation; general demand-order inertness is unproved.

The corrected per-query accounting is up to six sibling measure terms per target
for two selected suffixes, m substituted relations, and one m-parent node after
one ordinary closure. No checker-time scaling or adoption recommendation follows:
the precision and authority criteria already fail. Build time and acceptance's
extra lowering cannot be used as the required checker-cost comparison.

The next bounded question is whether whole-owned-root alternatives with one fixed
query suffix can use point-current structural authority and a source-root bound
without introducing a second path interpreter. This is an untested narrowing,
not a claimed fix. Range retention still needs separate capture availability,
conditional-fact and loop-generation justification.

## Reproducible precision discriminators

The prototype rejects the append below at FN-8. Removing only the later
`set selected = &full;` makes it accept. The baseline rejects both forms.
This complete helper isolates the function-global target-set dependence:

```wf
fn inspect(flag: own Bool) -> result: own unit pure {
  let left = slots_new::<u64, 1>();
  let right = slots_new::<u64, 1>();
  let full = slots_new::<u64, 1>();
  place_back(window: &full, value: 3_u64);
  let selected = if flag {
    give &left;
  } else {
    give &right;
  }
  place_back(window: selected, value: 7_u64);
  set selected = &full;
  return unit;
}
```

For Boolean composition, pass a reference joining two empty `Slots<u64, 1>`
to this helper. The prototype accepts the two separate requirements below.
Replacing them with only `requires band(empty, available);` rejects at FN-8;
replacing them with only `requires bnot(full);` also rejects. The baseline
rejects all three callers. No change to the callee's body is involved.

```wf
fn needs_empty(value: &Slots<u64, 1>) -> result: own unit reads(value.len), reads(value.cap) contract {
  define empty = deref(value).len == 0_u64;
  define available = deref(value).len < deref(value).cap;
  define full = deref(value).len >= deref(value).cap;
  requires empty;
  requires available;
} {
  let length = deref(value).len;
  let capacity = deref(value).cap;
  return unit;
}
```

The retained positive test
`selected_targets_subscript_and_integer_domain_share_the_route` creates two
`Slots<u64, 1>`, places one value in each, joins their references as `selected`,
then executes these exact statements:

```wf
  let observed = deref(selected)[0_u64];
  let quotient = 4_u64 / deref(selected).len;
```

The subscript is admitted, but division rejects at OP-2. The source test remains
expected-positive: 12 of the 13 focused controls pass and this one fails.

**Design suitability:** retain this experiment as evidence for deferral. The
query-only approach buys local precision, but the present general candidate does
not satisfy the required authority, composition and source-cost conditions.
