# Static privilege-definition gate decision

Status: D14 architectural research recommendation, 2026-07-15. Owner
selection is pending. This document authorizes no language, specification,
compiler, verifier, runtime, standard-library, capability-basis, or experiment
change.

## Decision question

Choose one static route by which xlang's compiler, runtime, or official core
implementation can define a semantic operation that ordinary source may call
but cannot define. The route must work when xlang ships no standard library,
must expose no writer-selectable privileged mode, and must not make a name,
path, attribute, symbol, contract, or command-line flag authoritative.

The production-language evidence is in
`STATIC-PRIVILEGE-MECHANISM-CENSUS.md`. This comparison does not select the
semantic operations that use the route.

## Recommendation

Select a **sealed compiler-embedded primitive registry** as xlang's sole
privilege-definition route.

At compiler-context construction, the compiler creates one built-in module
from a closed registry. Each registry row creates one declaration with:

- a compiler-owned semantic identity;
- a complete checked type, ownership, effect, trap, and target-availability
  contract;
- an abstract operation meaning used by the checker, constant evaluator,
  optimizer interface, and lowerer; and
- either a direct backend lowering or an exact runtime-body binding selected by
  that same semantic identity.

Ordinary source can import, alias, re-export, wrap, and call the public checked
declarations. None of those actions creates a new semantic identity. A source
module with the same name is not the embedded module, a source declaration with
the same signature is not an embedded declaration, and a native symbol with the
same spelling is not a registered runtime body.

The registry is an architectural object, not a proposed file format, keyword,
module name, or source spelling. It may be implemented as compiler data or code,
but it is never parsed from application or dependency source.

## One authority decision, several physical implementations

The authority decision is exactly:

```text
declaration identity was created from the compiler-embedded registry
```

Direct instructions, compiler-generated control flow, libcalls, runtime
helpers, OS entries, and target-specific sequences are implementation choices
behind an already-authorized declaration. They do not create additional
privilege routes. In particular:

- the lowerer dispatches on semantic identity, never a name or attribute;
- a runtime binder accepts only the exact semantic identity and the ABI/target
  contract fixed by its registry row;
- unsupported targets reject the operation rather than choosing a weaker
  implementation;
- constant evaluation either implements the same abstract meaning or rejects
  evaluation; and
- optimizer facts may be emitted only from the fixed abstract contract and
  verified implementation path, never from wrapper source or a runtime symbol.

This keeps one semantic admission route without pretending that every machine
operation has the same physical body.

## Ordinary safe-call model

The embedded declaration is a normal checked callable at use sites. Calls obey
the same typing, ownership, effect, and check rules as any ordinary function.
The only exceptional property is that the declaration's body meaning comes
from its compiler-owned identity rather than an ordinary xlang body.

An ordinary library may provide a higher-level safe abstraction only when its
ordinary body type-checks over these declarations. It receives no privileged
package status. A wrapper cannot widen effects, remove checks, install facts,
or fabricate ownership transitions beyond what the embedded declaration's
contract and the ordinary checker prove.

This separates public availability from definition authority and permits a
no-standard-library distribution: the compiler carries only the primitive
declarations, while containers, strings, resource wrappers, and other APIs can
remain ordinary checked libraries.

## Extension and review path

A new primitive requires a toolchain source change that adds one registry row
and discharges all applicable obligations:

1. state one abstract semantic contract, including failure, cleanup, effects,
   ownership transitions, traps, and target availability;
2. show why existing language semantics and existing registered primitives
   cannot derive it safely and without an unacceptable structural cost;
3. implement and test every required checker, constant-evaluation, direct
   lowering, runtime, and backend case;
4. prove or validate that each implementation refines the same contract;
5. account for every optimizer fact the operation can mint;
6. add negative tests for forged names, declarations, serialized artifacts,
   flags, attributes, symbols, and unsupported targets; and
7. receive hostile review before the row can ship.

An ordinary library release cannot add a row. A toolchain plugin, backend
plugin, runtime plugin, JIT payload, or command-line option cannot add a row.
Changing the registry is a new compiler release decision under the existing
repository and owner process, not a cryptographic authorization protocol.

## Architectural comparison

| Candidate | W3 authority boundary | P0/code shape | W1 surface | No-standard-library fit | TCB and extension path | Decision |
|---|---|---|---|---|---|---|
| Compiler-hard-coded public operations | Strong when each operation is compiler syntax/table data | Direct lowering is optimal; per-operation parser/checker paths can accumulate | Each special operation can add a visible rule or spelling | Strong | Compiler branches grow per operation; runtime still needed for some bodies | Viable but not selected as the general route |
| Compiler-only intrinsic declaration form | Strong only when declaration admission depends on toolchain-owned artifact identity | Direct lowering and inline wrappers are possible | Public calls can remain regular | Strong if declarations are embedded | A hidden source loader or special parser mode becomes additional TCB | Collapses to the recommendation when the artifact is embedded; otherwise adds an unnecessary admission path |
| Sealed compiler-embedded module/registry | Strong: compiler-created identity, not spelling | Direct lowering or exact runtime body; no inherent transition | One normal checked call model | Strong | One closed registry plus implementations; each extension is a compiler review | **Recommended** |
| Special official-core source mode | A public flag fails W3; an internal-only build action can pass | Direct lowering is possible | Ordinary calls can be regular, but the compiler has a second mode | Requires a build-only core artifact or ships a core library | Adds parser/driver mode, artifact provenance, bootstrap, and release-pipeline trust | Rejected: no independent need under the embedded registry |
| Exact native runtime imports as the primary route | Symbol/name matching fails W3; authorized declaration identity can pass | Mandatory transitions can cost; not all primitives require runtime state | Wrappers can be regular | Runtime-dependent | ABI registries and target bodies add TCB | Rejected as primary authority; retained only as a subordinate body option |

## Why the recommendation survives the comparison

The embedded registry retains the zero-transition path of hard-coded operations
without requiring a new grammar or writer-visible checker-rule spelling for
each public primitive.
It retains the declarative contracts and ordinary wrappers of intrinsic
declarations without a privileged source parser, bootstrap mode, or artifact
loader. It provides the declaration identity that exact runtime imports lack,
while allowing a runtime body only when the operation intrinsically requires
one.

The recommendation is Pareto-small in authority routes: deleting the embedded
identity leaves no static way to distinguish a privileged declaration from an
ordinary one; adding a special source mode or independent symbol/attribute
route supplies no required capability and increases W3 and TCB surface. This is
an architectural minimality claim about the route, not a claim that the future
semantic operation set is minimal.

## P0, W1, W3, and Rust delta

- **P0:** direct backend lowering has no inherent wrapper or privilege-check
  transition. A runtime crossing is charged only to operations whose physical
  implementation requires it. Fixed contracts give the optimizer stable facts,
  subject to the existing hostile-review rule for fact channels.
- **W1:** ordinary code sees one normal checked call model. There is no
  privileged dialect, unsafe block, attribute, or flag to teach or misuse.
- **W3:** a writer can call fixed safe operations but cannot mint the
  compiler-created declaration identity, select an official-core mode, install
  a contract, or bind an arbitrary implementation.
- **Delta over Rust:** xlang retains compiler-owned intrinsic semantics and safe
  wrapper composition while removing writer-accessible `unsafe`, nightly
  lang-item/intrinsic definition, and user-selectable internal attributes. The
  intended machine-code path remains direct, so the delta is W3 rather than a
  concession on P0.

## Explicit non-decisions

This recommendation does not decide:

- which storage, ownership-transition, atomic, OS, target, or runtime
  operations are irreducible;
- whether a particular operation is direct-lowered or runtime-backed;
- module, declaration, keyword, attribute, or diagnostic spelling;
- binary encoding, registry layout, versioning, or compiler data structures;
- a standard library, container design, runtime policy, package authority, or
  independently distributed privileged extension system; or
- any production implementation or specification change.

The next serial step is hostile review of the authority boundary. Public-basis
derivation remains blocked until that review passes.
