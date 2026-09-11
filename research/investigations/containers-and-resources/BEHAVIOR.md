# Behavior parameterization

D6 proposal against v0.56, for owner selection; full-source appendices are
linked. Replace them with executable witnesses after selection and implementation.

## Recommended surface

Extend [FN-5](../../../spec/kernel-spec.md#8-functions-generics-contracts)'s
deferred env-struct direction using ordinary values and source contracts:

```wf
contract Key<K: linear, E: linear> {
  fn hash(env: &E, key: &K) -> hash: own u64 effects;
  fn equal(env: &E, left: &K, right: &K) -> equal: own Bool effects;
}
conform Seed : Key<u64, Seed> {
  hash = seed_hash;
  equal = seed_equal;
}
```

A generic client writes `K: linear, E: linear + Key<K, E>` and
`E.hash(env: env, key: key)`; its caller writes
`find::<u64, Seed>(slots: &slots, key: &key, env: &seed)`.
The subject E is an explicit environment type, never implicit `Self` or a
receiver. `+` adds one source-contract obligation beside S37's mandatory class;
`linear` admits every element class without allowing generic disposal.
`E.member` selects only E's written bound. Prefer dot over the sole alternative
`E::member`, keeping `::` for explicit instantiation. Empty environments cost
no dictionary. [Section 15](../../../spec/kernel-spec.md#15-prelude-normative-counted)'s
`Int`/`Float` retain their canonical numeric bounds and closed conformer sets.

**Brand prerequisite, separately proposed:** FN-2 currently requires receivers
to declare every store region hidden inside a type argument. Remove that
redundant declaration requirement for an opaque T/E, preserving the full branded
type identity, STOR-5 confinement and explicit release capability. Hidden loans
and providers remain forbidden. Otherwise one fixed generic declaration cannot
accept arbitrary multi-store keys.

Allow region-only binding schemes, e.g. `conform ['s] Seed :
Key<Box<'s, Resource>, Seed> { hash = boxed_hash; equal = boxed_equal; }`.
No type/const blanket bindings. Coherence keys include the exact subject and
all instantiated contract arguments; reject any schemes whose finite structural
region patterns unify, including concrete/scheme overlaps. Match bound functions'
input type trees after substitution by FORM-8, fixing store brands there;
remaining loan binders match by FN-3 ordinal alpha identity. Undetermined region
arguments and all generic function arguments are explicit in `fn_bind` as
`member = function::<...>;`; none are inferred from a desired result.

## Checking and effects

Recommend **per-inhabited-instance checking**. Keep FN-2's symbolic spelling,
linearity and member-independent checks; defer member-dependent proof to the
concrete instance. Resolve one complete conformance, substitute the named
function, and check its ordinary call. Parameter/result modes, types and
region relationships must match exactly. Labels follow the interface and map
by ordinal; no coercion, implicit receiver or inferred type/const/behavior arguments.
Its own declared row and optional function `contract` supply the requires and
ensures. Prove each requirement at the call and verify each postcondition under
FN-9, including same-SCC withholding. `effects;` on the interface is an explicit
open row/contract slot, not a claim of purity, totality or any logical fact.
Legacy fixed-row source contracts keep their exact-row matching.

Write later-supplied effects as erased signature splices:
`reads(slots), effects(E.equal(env: env, left: slots[], right: key))`.
A splice references a selected member or explicitly instantiated ordinary
helper's **declared row**, never its body. Named arguments are state-origin
images, not evaluated expressions: `path` supplies its complete incoming state
origins and composes static field suffixes; `path[]` collapses contained dynamic
elements or enum payloads to that enclosing path; `unit` supplies fresh state.
Repeated splices union alternative origins. Projection uses EFF-2's concrete
mode/copy framing and most-precise static paths: an own copy scalar frames out,
an incoming owned struct retains its field states. Argument evaluation remains
a separate body contribution. Expand reads, writes and allocates, then apply
EFF-2's exact check in both directions, including releases; any row mismatch
rejects. These annotations never narrow actual loans
or CALL-6/ENT-5 kills.

Checking once against a bound needs fixed rows/contracts and refinement rules:
earlier diagnostics and amortized proof, but stronger concrete facts require
rechecking. Instance checking reuses today's checker with repeated work and
instance-specific diagnostics. Direct dispatch alone establishes no performance parity.

## Finite checking and rule delta

Extend FN-6's conservative cycle rule across type, const and behavior arguments,
member bindings, row references and nominal/contract dependencies. Build the
finite declaration graph first, including compatible bindings. Inside a
recursive SCC, generic declarations have matching ordered parameter kinds and
forward them unchanged; nongeneric bridges cannot inject new generic arguments.
Reject growing edges such as `f::<T, n + 1, E>` or `f::<Box<T>, n, E>` with the
cycle named. Intern instances renaming only bound region identifiers, preserving
actual store-brand equality, store class and outlives constraints. Recursive
rows use the **least fixed point**, unioning declared literals and projected
splices over the finite admissible formal paths/regions. No timeout or fuel
selects acceptance. Acyclic instance multiplication remains FN-2's
[recorded compile-cost question](../decision-workflow/RULE-GROUNDS.md#functions-and-contracts).

| Rule / grammar | Proposed delta |
| --- | --- |
| FN-2 / PROV-6 | Class plus source bound; explicit arguments; opaque-brand prerequisite. |
| FN-3 | Generic interfaces, open boundaries, exact signatures and coherent schemes. |
| FN-5 / TYPE-6 | Qualified member selection becomes an ordinary direct function instance. |
| FN-6 | Argument-preserving cycles include const, behavior and indirect dependency edges. |
| FN-8 / FN-9 | Concrete member obligations; no assumed law or new fact source. |
| EFF-1 / EFF-2 | Signature row splices, origin images and finite least closure; retain exactness. |
| CALL-6 / ENT-5 | Use selected ordinary boundary; retain resolved-place projection and kills. |
| GRAM-2 / FORM-8 | `+` bounds, open `fn_sig`, region schemes and `fn_bind` arguments. |
| GRAM-5 / GRAM-11 | `TYPEID . IDENT` callee; member-declared argument names/order. |
| FORM-1/2 / DIAG-1 | Canonical forms/diagnostics; one new keyword, `effects`. |

## Full source and boundary

[Owning map](behavior-map.wf): D2 enum slots and Box payloads; generic K,
collision/replacement/removal, tombstones, reuse, allocation refusal, rehash and
concrete cleanup. Displaced ownership returns explicitly. Rehash rotates the
source through take/replace/place; it does not reproduce D2's incremental budget.
[Priority queue](behavior-priority.wf): generic linear T, shared comparison,
two-state push/pop and explicit concrete instantiation. A temporarily removed
tail enables single-place exchanges without a vacant T. Both retain owners;
the extra moves and descriptor writes need measurement. Dynamic-index swaps
remain a separate LIV-2/OWN-7 limitation, not a behavior amendment.
For `push`, `heap: value` is the inserted owner's state image (`value.rank`
for `Item`, empty for a copy scalar).

No hash/equality/order laws are assumed: inconsistent behavior affects contents,
not ownership or proved bounds. Shared inputs grant no mutation or provider
capability. Custom interfaces needing either must expose it as ordinary inputs.
No function values, dictionaries, dynamic dispatch, inference, defaults,
higher-kinded behavior or specialization. Next-goal evidence is instantiated
checking, failure diagnostics, hostile behaviors, direct-call IR and retained
D1/D2 timing controls; these listings are not benchmark results.

Local sanity check with the D5 compiler (`85fce15f`): literal direct-call
expansion compiled and ran the map; priority passed semantic checking but hit
backend `InvalidIr`. This does not validate the proposed generic/row mechanism.
