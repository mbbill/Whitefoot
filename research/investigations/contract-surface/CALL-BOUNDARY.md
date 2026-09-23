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

## Result naming alternatives

The present [contract decision](../../../design/language/checks-and-proofs/requires-entry-contract.md)
puts every result name in the signature so that postconditions can refer to
the whole result separately from a routed payload. Preserve that distinction;
compare the location and introduction of the names, not their proof authority.

| Alternative | Complete rule | Benefit | Cost or reason not selected |
|---|---|---|---|
| Keep signature names | Every result, including unit, has `name: T`. | All names and types appear together; no new declaration form. | Introduces a proof candidate even when there is no result relation. A name that cannot be used in the body resembles a runtime output binding. |
| Selectively omit a header name | Permit `-> T` as well as `-> name: T`, or require omission for unit or unused results. | Small local edits and concise simple headers. | The optional version gives the same boundary two annotation forms; type- or use-dependent versions make spelling depend on substitution or on other clauses. This study does not select that relaxation. |
| Implicit result names or indices | Give one result a built-in name and multiple results fixed indexed names. | No separate name declaration. | Removes descriptive names from cross-result relations or adds a second mechanism for them; integer result indices expose ordinal bookkeeping directly in proof expressions. |
| Bind results per ensures clause | Each clause introduces its own result pattern or result-name list. | Names are local to one relation. | Repeats the same ordinal mapping across independent clauses; clauses comparing two results still need both identities. The v0.32 single-relation selector is not sufficient for today's shared contract block. |
| Declare names once in the contract | Headers always contain types; a contract may declare `results (name0, name1, ...);` before its definitions and clauses. | Locates proof-only aliases with their only consumers while retaining descriptive names and one shared ordinal mapping. | Adds a proof declaration and the `results` keyword. Result-bearing contracts gain a declaration line; a list still names every ordinal, including ones no clause uses. |

Recommend the last form together with removing written `own`. The reason is
the separation of the callable's type boundary from proof-only bindings:
FN-4 already compares results by ordinal and type, and FN-9 consumes the
names only when forming postconditions. This is not selected by how often
contracts occur in the current corpus or by implementation effort. Retaining
all header names remains the viable simpler-grammar alternative if keeping
names adjacent to their types is valued more than that separation.

Other languages demonstrate the choices, without deciding Whitefoot's:

- [Rust's function grammar](https://doc.rust-lang.org/reference/items/functions.html)
  writes parameter types and an unnamed return type. Its omitted unit output
  and other inference rules are not part of this proposal.
- [WhyML's grammar](https://why3.org/doc/syntaxref.html) offers named results,
  `ensures` using `result`, and `returns` patterns. The relevant alternative
  is binding a proof name in a postcondition; importing all these spellings
  would not satisfy this study's single-form criterion.
- [Dafny's quick reference](https://dafny.org/dafny/QuickReference) gives methods
  named output parameters that the body assigns. Those names have a runtime
  role that Whitefoot's proof-only result aliases do not have.

## Proposed declaration and binding rules

These are proposed rules, not amendments to the active specification yet.

1. A value parameter is `name: T`; references remain `name: &T` and
   `name: &[T]`. A result is written as `-> T`, or `-> (T0, T1, ...)` for two
   or more results. Every result is owned. `-> unit` and `return unit;`
   remain explicit. `own` is not an optional qualifier.
2. All ordinary functions, raw function formals, interface members and PRE-1
   declaration spellings use that same boundary. No reference-valued generic
   argument, reference result, tuple expression or implicit result variable
   is introduced.
3. A contract may start with one `results (name0, name1, ...);` declaration.
   A present declaration names every result ordinal exactly once in result
   order; count mismatch or duplicate names is an FN-9 formation error at
   the declaration. One result also uses parentheses. Sparse lists and
   placeholder names have no separate syntax.
4. The names are proof candidates, subject to the current result-candidate
   reservation, distinctness and per-clause FN-9 admission rules. They must
   differ from parameters and contract definitions. They are unavailable in
   `define`, `requires` and the body; a body local can reuse a result name
   as it can today. Eligibility as an integer or measured datum is unchanged.
5. A routed clause continues to use `when name is Ok(value: payload):`, or
   the existing omitted-owner route when exactly one declared ordinal has
   the eligible enum type. The former requires a declared result alias;
   the latter does not. Its payload has the existing clause-local scope and
   freshness rules. The routed whole-result alias stays unavailable in that
   clause; aliases for other ordinals remain available. Two eligible enum
   results still require an explicit, unambiguous owner.
6. Without a `results` declaration, the contract has no named whole-result
   candidates. It can still state requirements, exit-state postconditions,
   and an unambiguous omitted-owner Result route. Presence of the declaration
   is an explicit binding choice available in every contract, never inferred
   from an instantiated type, use count or proof success. Unused aliases are
   legal; FN-8 still requires at least one requires or ensures clause, so a
   names-only block is rejected. There is no named-header alternative.
7. Contract normalization still removes alias spellings and retains result
   ordinals, types, routes and provenance. A formal and actual may choose
   different aliases; FN-4 still checks the same relation refinement. Body
   result construction, return transfer, destructuring at calls, proof
   erasure and result-fact transport are unchanged.
8. `results` becomes a fixed grammar word and therefore ceases to be an
   IDENT under FORM-3. With no remaining fixed grammar atom `own`, `own`
   becomes an ordinary IDENT under that same rule; no historical keyword
   reservation is retained. This lexical consequence is part of the proposal.

The optional declaration introduces names, like a proof definition; it does
not let the writer choose whether to annotate the same result type. A header
always writes a type and never writes a result name. Contract declarations
have a fixed order, and all other canonical formatting rules remain in force.

The complete changes to the affected grammar productions are:

```text
fn_decl      := "fn" IDENT generics? "(" param_list? ")"
                "->" (type | "(" type ("," type)+ ")")
                effects contract_block? "{" doc? stmt* "}"
fn_sig       := "fn" IDENT "(" param_list? ")"
                "->" (type | "(" type ("," type)+ ")")
                effects contract_block?
param        := IDENT ":" (type | "&" (type | "[" type "]"))
contract_block:= "contract" "{" result_names? contract_define* requires_clause* ensures_clause* "}"
result_names := "results" "(" IDENT ("," IDENT)* ")" ";"
```

Retire `result_binding`, `rtype` and `mode`. `type` retains its current
definition and cannot start with `&`; reference-result spellings fail the
grammar. A single parenthesized result type is also outside the grammar.

## Complete-function comparisons

The candidate definitions below correspond to maintained source definitions.
Their linked files supply the current full declarations, bodies and callers;
no candidate is claimed to compile in the unchanged production compiler.
The comparisons preserve bodies, calls, contracts and result ordinals while
changing the stated boundary forms. Nominal and prelude dependencies retain
their existing meanings.

### Scalar computation with no result contract

The [SHA-256 helper and its caller](../../../tests/programs/sha256_abc.wf)
currently name both results and write `own` on each value parameter/result.
Neither helper uses its result name in a contract. The candidate is:

```text
fn xor_three(x: u32, y: u32, z: u32) -> u32 pure {
  let first = ixor(x, y);
  return ixor(first, z);
}

fn sha256_small_zero(word: u32) -> u32 pure {
  let rotate_seven = irotr(word, 7_u32);
  let rotate_eighteen = irotr(word, 18_u32);
  let shift_three = ishr.wrap(word, 3_u32);
  return xor_three(x: rotate_seven, y: rotate_eighteen, z: shift_three);
}
```

The ordinary call retains labels; `ixor` and the other OP-1 operations retain
their separate positional grammar. Value/reference classification reads only
the header, even for `T` rather than a primitive type.

### One result shared by several clauses

The [signed clamp](../../../tests/conformance/cases/fn9-pos-two-signed-result-bounds.wf)
keeps its two independent relations and its result name:

```text
fn clamp_range(value: i32, floor: i32, ceiling: i32) -> i32 pure contract {
  results (result);
  requires value >= floor;
  requires value <= ceiling;
  requires floor >= -100_i32;
  requires ceiling <= 100_i32;
  ensures result >= -100_i32;
  ensures result <= 100_i32;
} {
  return value;
}
```

Keeping signature names would instead write `-> result: i32` and omit the
`results` line; per-clause binding would repeat the declaration for both
ensures clauses. The proposed line is an alias declaration, not a return
assignment or a stored value.

### Generic measured result, routed result list and requires-only unit

These three complete definitions come from the same
[two-result conformance program](../../../tests/conformance/cases/call4-pos-route-names-a-result-ordinal.wf):

```text
fn filled<T: copy, const n: u64>(value: T) -> Slots<T, n> pure contract {
  results (result);
  ensures result.len == n;
} {
  doc "Builds a run of n slots, every one holding a copy of value; the copy bound is what admits the bare repeated use.";
  let block = array_filled::<T, n>(value: value);
  let built = slots_from_array::<T, n>(values: block);
  return move built;
}

fn probe(taken: Slots<u8, 8>) -> (Result<u64, Overflow>, u64) pure contract {
  results (outcome, spare);
  ensures when outcome is Ok(value: reported): reported == taken.len;
  ensures spare == taken.len;
} {
  doc "One route names the enum ordinal it applies to; the unrouted clause names the other.";
  let measured = taken.len;
  return Ok<u64, Overflow>(value: measured), measured;
}

fn needs_eight(bound: u64) -> unit pure contract {
  requires 8_u64 <= bound;
} {
  doc "Needs a caller proof about its argument.";
  return unit;
}
```

The caller still writes `let (outcome, spare) = probe(taken: move run);`.
Those body names are unrelated to the callee's proof names; result ordinals
make the connection. `move` on the generic owned return and on the call is
unchanged. The `copy` bound, measured result admission, routed payload and
requirement proof obligations are also unchanged.

### Unit result with an exit-state relation

The [owning-map helper](../../../tests/programs/containers/owning-behavior.wf)
has an ensures clause about written-through storage, not its unit result.
The full candidate definition needs no result alias:

```text
fn key_fill<K>(slots: &Box<Slots<Slot<K>>>, count: u64) -> unit writes(slots) contract {
  requires deref(slots).inner.len == 0_u64;
  requires count <= deref(slots).inner.cap - deref(slots).inner.len;
  ensures deref(slots).inner.len == count;
} {
  doc "One vacant slot per position, under a counted header that carries the length and the remaining free slots.";
  for (
    index in 0_u64..count,
    invariant filled: deref(slots).inner.len >= index,
    invariant bounded: deref(slots).inner.len <= index,
    invariant spare: deref(slots).inner.cap - deref(slots).inner.len + index >= count
  ) {
    let vacant = Vacant<K>();
    place_back(window: &deref(slots).inner, value: move vacant);
  }
  invariant complete_min: deref(slots).inner.len >= count;
  invariant complete_max: deref(slots).inner.len <= count;
  return unit;
}
```

No rule asks whether the return is unit before permitting this spelling.
An exit-state-only contract on a non-unit function uses the same form.
The reference marker, write effect and entry/exit-state distinction stay
explicit and unchanged.

### Formal result types and resource consumption

The [formal-result case](../../../tests/conformance/cases/fn4-pos-formal-owned-result-transfer-2.wf)
retains generic result substitution and the same FN-4 boundary:

```text
interface Factory<T: drop> {
  fn make(value: T) -> T pure;
}

fn retain(value: Slots<u8, 4>) -> Slots<u8, 4> pure {
  doc "An owned formal result may be a newly constructed value of the exact declared type; the unused incoming owner takes its compiler-derived release [STOR-3].";
  return slots_new::<u8, 4>();
}

binding Identity : Factory<Slots<u8, 4>> {
  make = retain;
}
```

The [deque's must-consume helper](../../../tests/programs/containers/deque-program.wf)
still consumes the whole ticket and transfers its payload explicitly:

```text
fn deque_ticket_consume(env: &DequeObservation, value: DequeTicket) -> unit writes(env) {
  let DequeTicket(payload: payload) = move value;
  deque_owned_consume(env: env, value: move payload);
  return unit;
}
```

`DequeTicket` is a `nodrop` owner in the linked source. Its parameter remains
a value parameter with the same consumption obligation; deleting the word
`own` does not make the value copyable, droppable or implicitly borrowed.

## Named operands and the error boundary

Retain GRAM-8/10/11: every ordinary call argument, construction field and
match field label is written in declared order, including a single field.
Changing mode spelling supplies no reason to delete these checks. Function
formals continue to supply their own parameter labels at the call; actual
parameter names do not replace them.

For `fn difference(left: u32, right: u32) -> u32 pure`, whose body returns
`left -wrap right`, compare:

| Candidate call | Judgment and reason |
|---|---|
| `difference(left: 7_u32, right: 2_u32)` | Well-formed labels and order; returns 5. |
| `difference(right: 2_u32, left: 7_u32)` | Rejected: labels are out of declared order. |
| `difference(left: 7_u32, left: 2_u32)` | Rejected: duplicate label rather than the second parameter's label. |
| `difference(left: 7_u32)` | Rejected: missing second argument. |
| `difference(lhs: 7_u32, right: 2_u32)` | Rejected: wrong label. |
| `difference(left: 2_u32, right: 7_u32)` | Well-formed and returns 4294967291; labels cannot prove intended values. |

The current construction decision overstates this protection when it says
names lift a same-typed transposition to a rejection without identifying what
was transposed. Propose correcting that ground to label/order mismatches and
retaining the value-swap limitation. No named-operand rule changes.

Allowing reordered named operands would remove the order check and canonical
order; allowing positional or same-name shorthand operands would remove a
written label check or add another source form. None is selected here. This
does not claim labels prevent all logic errors or establish an empirical
authoring benefit.

## Implementation boundary and remaining validation

The proposal changes source binding location and grammar, not the proof
domain, ownership representation, effects, ABI or Result evidence closure.
Normalize declared contract aliases to existing result ordinals before
forming relation templates. A missing alias list must be represented as
absent, not filled with invented source names or hidden runtime locals.

An implementation must update GRAM-2/3 and its generated syntax consumers,
FORM-3's terminal inventory, TYPE-5/6, FN-1/4/8/9, CALL-4, affected DIAG-1
ownership and locations, PRE-1 declaration records, examples, writer patterns,
and conformance evidence together. FN-9 owns count, distinctness, scope and
route errors; grammar owns malformed declarations. All source and prelude
signatures must use the same formation path, including raw function-kind
parameters and interface members. Internal owned-value terminology may
remain; written `own` is being removed from source signatures.

Implementation acceptance must include positive and negative pairs for value,
reference and range-reference parameters; single and multiple results;
missing, repeated, misplaced and wrong-arity alias declarations; generic
substitution; result-alias exclusion from requires/define/body; freshness and
legal body-name reuse; routed whole-result exclusion and ambiguous routes;
formal/actual renaming with stronger and weaker contracts; PRE-1 summaries;
copy, drop and must-consume transfer; and keyword reservation. Existing
label/order controls and same-typed value-swap acceptance remain unchanged.
Run the canonical `make check` after implementing the amendment. A grammar
probe alone cannot establish any of those semantic paths.

The structural opportunity is to remove mandatory proof names from callable
type records without changing result identity. It is in scope. Broader
contract-expression changes and ownership-transfer syntax are deferred to
their existing TODO groups: neither is required to implement this boundary,
and each needs its own semantic examples. Writer productivity and compile-time
improvements remain unmeasured and are not claimed or used for selection.

## Proposed tree revision

Three full-node replacements are proposed in `design/amendments/`; the live
tree is unchanged. Applying all three would retain 80 nodes and depth 3,
with two additional Decision fields and no additional Rejected entries.

- `language/surface-form`: replace the boundary-annotation part of the body
  binder decision and add the value/reference spelling decision. The current
  ground that boundary modes cannot be reconstructed no longer justifies a
  separate `own` marker. Body inference and all other surface decisions stay.
- `language/checks-and-proofs/requires-entry-contract`: replace mandatory
  signature result names with the contract-local alias declaration; retain
  the separate whole-result/payload distinction in its own decision. Remove
  the absence of a result name from the reason for rejecting the old block
  form, because erased grouping and shared abbreviations justify the block
  independently of where result aliases are introduced. Clause checking,
  state relations, refinement and proof erasure do not change.
- `language/surface-form/construction-form`: correct the transposition ground
  in the first Decision and its matching Rejected entry. It covers labels and
  their order, not every exchange of values of the same type. Field/call rules,
  the single-field rule and the consume rest marker stay unchanged.

This study proposes a source-language change, not its implementation. No
active specification, released archive, compiler, test or conformance verdict
is edited. Implementation-dependent guidance remains current until those
rules change together in a subsequent implementation step on this PR.
