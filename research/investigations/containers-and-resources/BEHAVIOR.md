# Behavior parameterization: D6b comparison

Proposal against [v0.56](../../../spec/kernel-spec.md), not a language change.
Supersedes rejected D6. Criterion: function-centric, fixed interfaces with
minimal mechanisms and synchronized declarations, serving the constitution's
[delegation/performance objectives](../../../docs/constitution.md).

## Evidence from seven families

**Ada/SPARK.** Formal subprograms accept named actuals; formal packages reuse
whole parameter groups. Profile matching is not WF proof checking. SPARK requires a generic's copied `Global`/`Depends` specifications to
work for every instance; it explicitly suggests restricting actual subprograms
to a fixed global set. SPARK violations can still appear at instance analysis.
Ada rejects recursive generic instantiation, including indirect nesting. Growth: Ada 83 formals gained formal packages in Ada 95, then aspects; import grouping, not the package/overload system. [1]

**ML.** Signatures name reusable type/value interfaces; structures supply them;
functors accept structures. Ordinary function types supply neither WF effects
nor proved pre/postconditions. SML97 elaborates module syntax rather than
requiring recursively generated monomorphic machine instances; functions remain
runtime values, so direct calls are not a language guarantee. SML97 revised sharing/opaque matching; OCaml grew higher-order, recursive
and first-class modules. Retain signature reuse, not module evaluation. [2]

**Zig.** `comptime` parameters accept functions/types; namespace structs group
operations without a separately declared interface. Bodies are checked for
selected arguments; assertions/reflection replace reusable formal contracts,
with no declared WF effect row. Unrestricted compile-time evaluation is bounded
by a configurable backwards-branch quota, not structural termination. The 0.4.0 comptime core coexists in 0.15.2 with reflection, inline expansion
and executable diagnostics; it lacks WF's determinate, fuel-free proof boundary. [3]

**Austral.** Linear universes coexist with Haskell-style typeclasses: signatures
are grouped, instances supply implementations, and compilation monomorphizes
method calls. Its documented function signatures provide no WF effect or
pre/postcondition calculus; linearity alone supplies neither. The reviewed spec
and compiler account do not establish a fuel-free finite-instantiation rule.
Longitudinal growth evidence is insufficient, but instance uniqueness,
overlap/orphan rules and resolution are already costs WF can avoid. [4]

**C++.** Concepts reuse argument constraints; policy types or constant function
arguments supply behaviors. Templates specialize direct calls, but concepts
constrain well-formed operations, not exact effects or machine-proved contracts;
`noexcept` is only about exceptions. Recursive instantiation/constant evaluation
has implementation limits, not WF's termination discipline. Historical layering
from templates/specialization through SFINAE, concepts and constexpr illustrates
how inference, overload choice and programmable formation accumulate rules.
C++26 contracts do not turn concepts into WF proofs. [5]

**Odin.** Explicit polymorphic type/constant parameters accept concrete procedures;
procedure types repeat signatures and procedure groups name overload sets, not
reusable behavioral interfaces. `contextless` removes an implicit context pointer,
not side effects; `where` checks are not WF pre/post proofs. Its overview does
not specify a finite-instantiation acceptance criterion. Starting as a Pascal
clone in 2016, it now combines explicit/implicit polymorphism, specialization,
`where` and procedure groups while still rejecting methods. Function-centric
syntax alone does not prevent generic-system growth. [6]

**Go.** Constraint interfaces reuse method/type-set requirements, satisfied by
types rather than separately supplied free functions; function arguments are
runtime values. There are no WF effect/proof contracts. The compiler detects
expanding type-flow cycles, a useful structural finiteness example; Go 1.18's
implementation uses shape sharing and dictionaries, not guaranteed direct calls.
Growth from Go 1.18 type sets to Go 1.20's `comparable` satisfaction exception
shows the interaction costs of retrofitting existing interfaces. [7]

## Three options on identical code

Complete, separate proposed-source units: [A: baseline](behavior-baseline.wf),
[B: nominal-as-shape](behavior-nominal.wf), [C: expanded/comptime](behavior-expanded.wf).
Each contains identical enum-slot owning-Box `find`/`put` and owning-element
priority `push`/`pop`, complete helpers/contracts and explicit instantiations.
Only parameter expansion, qualification and naming differ. These source
appendices replace neither executable D2 growth nor D5 timing controls; replace
them with the selected executable witnesses after implementation.

| Constraint / cost | A: shape + group | B: nominal + group | C: function-kind only |
| --- | --- | --- | --- |
| Free functions; no Self or dispatch | Yes | Parameter projection only | Yes |
| One semantic core; direct call | Function substitution | Same | Same |
| New top-level kinds / binder forms | 2 / 2 | 1 / 2 | 0 / 1 |
| New productions / keywords | 4 / 2 | 3 / 1 | 1 / 0 |
| Fixed row; explicit instantiation | Yes, boundary below | Same | Only with same boundary |
| Reused formal / actual lists | Both named | Both named | Both repeatedly written |
| Arbitrary functions / types | Ordinary signatures; limits below | Same, plus nominal dependency | Same; no reusable interface |
| Container identity includes behavior | `Map<SeedKey>` | `Map<SeedKey>` | `Map<u64, Seed, fn seed_hash, fn seed_equal>` |
| No-container sort | `sort<Order<T,E>>` | Dummy struct or depend on Queue | Repeat comparison signature |
| Laws / termination / region safety | None assumed; rules below | Same | Same; no arbitrary CTFE |
| Likely growth pressure | Group nesting, defaults | Fake types, header extraction, type projection | Signature aliases, inferred requirements |

**A** uses the owner's `shape Key<K: linear,E: linear>`,
`group SeedKey : Key<u64,Seed>`, `fn find<Key<K,E>>`, `Key::hash`, and
`find::<SeedKey>`. `Key<K,E>` forwards the whole group; the full
`Key<K1,E1>::hash` disambiguates distinct written applications before
substitution, even if their final types coincide. Two identically written
applications need separately named raw function formals: no pack aliases are
proposed. A group is an abbreviation,
not a type; equal expanded vectors give identical container types. Declaration
pack positions bind fresh ordered type/const names; forwarding positions name
existing bindings. Generated function-formal identities are hygienic: `Key::hash`
cannot capture `put`'s local `hash`.

**B** puts those function formals on `struct Map`; `fn find<Map<K,E>>`
imports its generic header, and `Map::hash` selects a formal, never a method.
`Map<'s,Map<K,E>>` explicitly forwards that header into a nominal instance;
header projection excludes the storage fields and nominal region list.
This saves `shape` but makes unrelated algorithms depend on a storage
abstraction or invent a fieldless struct. Types gain a second meaning.

**C** writes `fn hash(...) ...` inside every generic parameter list and passes
`fn seed_hash` at every application. It retains FN-2's per-instance checking.
A truly Zig-like version without formal rows/contracts violates the fixed-row
constraint; this restricted version retains them, but loses declaration-time
group validation and signature reuse.

## Shared semantic boundary and exact deltas

**Fixed does not mean inferred later.** At a formal call, use the formal's
written row and contracts, even after binding a narrower actual. Match modes,
types, ordered results and alpha-renamed regions exactly; require each actual
effect path to be covered by a formal path in the same category (prefix coverage,
allocation brands equal). Match ordered requires/ensures structurally after
binder/region normalization and definition substitution; no implication solver. A group checks all bindings atomically at declaration.
Generic actual functions must have all type/const/function arguments supplied;
region-polymorphic ordinary signatures remain available.

This **changes EFF-2's abstraction boundary**: exact means the complete syntactic
row using declared formal interfaces, not the narrower selected bodies.
For example, formal `reads(env,key)` remains so when its actual reads only
`env.salt,key`; no dummy reads or runtime adapter are emitted. Check generic
row attribution at the symbolic template, retaining formal state paths even
when a later copy instantiation would frame them out. Instance rechecks preserve that public row, rejecting additional effects;
ordinary call projection frames fresh state out. Heap `push` names read/write
support of the installed incoming `value`.
If “exact” instead requires re-derived concrete-body equality, the baseline's
subset binding cannot meet it; require equality instead, losing narrower
implementations.

**Rows alone are insufficient.** Both identity and fresh-constant implementations
can match `fn f(x: own Item) -> y: own Item pure`; reading `y.rank` later
attributes to incoming x only for identity. Propose one additional FN-1 rule:
non-copy results get a finite ceiling of compatible owned input roots,
transferable unique roots overlapped by declared writes, and fresh state.
Unwritten exclusive exits retain entry routing; written exits also admit entry
routing, transfers and fresh state. Returned copies are fresh; writing copy fields
preserves storage identity. Compatibility uses signature types, with opaque T
conservative. Exclude shared roots and allocator-to-allocation ancestry.
This bounds existing routing, not owner identity. Template effect
attribution uses that fixed ceiling; checked actual routing must refine it.
This unimplemented, unvalidated rule sacrifices field precision/independence.
Using actual routing instead is
smaller, but rejects otherwise matching functions whenever the wrapper's fixed
row differs. Hash/equality/comparison examples alone would miss this cost.

Keep FN-2/FN-9 instance checks for ownership/proofs, actual FN-1 target summaries
and recursive-summary withholding; use only formal proof contracts. FN-8 proves
requires, FN-9 verifies actual ensures, CALL-6 kills overlapping fixed-row support
before publication. `&uniq` bare/`entry(p)` measures denote exit/entry, including
fields and multiple results. Shared/view outputs keep FN-1 loan-origin bounds;
FORM-8 alpha-matches quantified formal loan regions, never extends loans.
Captured brands remain exact: `group G['s] : Key<Box<'s,Resource>,Seed> ...`.

All options expose FN-2's existing arbitrary-type limit: receivers must name
every store brand hidden in K. To support arbitrarily many such brands, propose
retaining K's complete opaque branded identity and STOR-5 confinement without
requiring redundant receiver binders. This is a proposed FN-2 delta, not current support. Hidden loans/providers remain excluded;
ordinary directly written loan/provider parameters, owned results and multiple
results are available on function formals. No higher-rank type-polymorphic
function actuals, implicit environment capture or behavior construction code.

**Finiteness:** resolve finite written bindings/forwarding edges before expansion.
Extend FN-6 to function and nominal-instantiation dependency cycles, the
complete type/const/function argument vector, and indirect formal-call edges: each edge within a recursive component must forward that
vector unchanged, modulo region alpha-renaming. Reject constructed/permuted
arguments on a cycle, including nesting a specialized function actual.
This also rejects `Grow<T>` containing `box<Grow<box<T>>>`; TYPE-2 alone
does not ensure finite nominal expansion. Acyclic source/header expansion is
finite; cycles create no new instance keys. No CTFE or fuel. This proves
termination only: acyclic fan-out can still be exponential under existing FN-2.
All three options need a bounded-cost admission/proof-reuse design to meet the
constitution; that shared prerequisite is unresolved, not discharged by timing.

**Grammar inventory** (new production counts, not net counts; reused names count
as modifications): new binder forms are function-kind (all) and pack (A/B).
All options add `function_arg`, modify `gparam`, `targ`
and `fn_sig`; `fn_sig` gains fn_decl's result list/contract and loses its trailing
semicolon. A adds `shape_decl`, `group_decl`, `pack_use`; B adds only the latter
two. A/B also modify `item`, `fn_bind`, `callee`; type syntax already parses
pack applications, with a kind check before expansion. C needs no such changes.
The exact new right-hand sides are:

```text
function_arg := "fn" callee ("::" targs)?
pack_use := TYPEID targs?
shape_decl := "shape" TYPEID generics? region_params? "{" doc? (fn_sig ";")* "}"
group_decl := "group" TYPEID region_params? ":" pack_use "{" doc? fn_bind* "}"
```

`gparam` adds `fn_sig` (A/B also `pack_use`); `targ` adds `function_arg`.
A/B's `callee` adds `pack_use "::" IDENT`; `fn_bind` becomes
`IDENT "=" callee ("::" targs)? ";"`. Only `shape`/`group` are new keywords.
Remove `contract_decl`, `conform_decl`, `law`, `law_arg`; C also removes unused
`fn_bind`. Keep `contract` for proof blocks; Int/Float become built-in bounds.
Rules: GRAM-1 permits abbreviation expansion with retained source nodes;
GRAM-2/3/5 and TYPE-5/6 cover syntax/kinds/resolution;
FN-1 adds the formal routing ceiling; FN-2–6 cover binding/checking/termination
and retire source laws;
EFF-1/2 fix formal attribution; FN-8/9, CALL-6, MSR-3 and FORM-8 reuse the above
contract/region judgments; PRE-1/section 15 retain numeric bounds.
Strong-LL(2) selector generation remains a required implementation check.

**Recommendation: A**, conditional on fixed formal rows and resolution of the
shared compile-cost prerequisite. It requires the FN-1 ceiling, not just
desugaring, but removes synchronized signatures without type-owned functions;
B saves one declaration kind at a lasting conceptual cost, C saves syntax by
moving that cost to every library layer. Initially keep groups flat, with no
inheritance, defaults, inference, overload choice or group-producing functions.
Zero dispatch/ABI overhead is predicted, native parity unmeasured: broad
rows can lose parallel independence; single-place heap exchanges cost extra
moves. Next: expansion IR, retained-helper timings, adversarial behaviors,
branded keys and two-state mutation; D5 scalar results do not cover these.

## Sources

Primary sources checked 2026-09-11; no cross-language timings.

[1] Ada [instantiation rule](https://ada-rapporteur-group.github.io/ARM/Ada_2012/RM-12-3.html), Ada 95 AARM [12.6](https://www.adaic.org/resources/add_content/standards/95aarm/AARM_HTML/AA-12-6.html), [12.7](https://www.adaic.org/resources/add_content/standards/95aarm/AARM_HTML/AA-12-7.html); AdaCore [SPARK RM 27.0w, §12](https://docs.adacore.com/spark2014-docs/html/lrm/generic-units.html).
[2] Milner et al., [SML97 Definition, §§3/5/7 and appendix G](https://smlfamily.github.io/sml97-defn.pdf); [OCaml 5.3 generative functors/extensions](https://ocaml.org/manual/5.3/generativefunctors.html).
[3] Zig official language references [0.4.0](https://ziglang.org/documentation/0.4.0/#comptime), [0.15.2 comptime and quota](https://ziglang.org/documentation/0.15.2/#comptime).
[4] [Austral specification](https://austral-lang.org/spec/spec.html), [author's compiler design, 2023](https://borretti.me/article/design-austral-compiler).
[5] C++ draft [template parameters](https://eel.is/c++draft/temp.param), [constraints](https://eel.is/c++draft/temp.constr), [implementation limits](https://eel.is/c++draft/implimits), [contracts](https://eel.is/c++draft/dcl.contract).
[6] Odin official [overview](https://odin-lang.org/docs/overview/), [history/FAQ](https://odin-lang.org/docs/faq/).
[7] Go [specification](https://go.dev/ref/spec), [instantiation-cycle checker](https://go.dev/src/go/types/mono.go), [Go 1.18 dictionary design](https://go.googlesource.com/proposal/+/master/design/generics-implementation-dictionaries-go1.18.md), [Go 1.20 comparable change](https://go.dev/blog/comparable).
