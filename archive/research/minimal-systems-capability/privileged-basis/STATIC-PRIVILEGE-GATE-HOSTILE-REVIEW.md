# Static privilege-definition gate hostile review

Status: D14 adversarial architectural review, 2026-07-15. Review target:
`STATIC-PRIVILEGE-GATE-DECISION.md`. This review can support an owner research
selection; it authorizes no implementation or production change.

## Verdict

**PASS at the architectural recommendation level, production fail-closed.**

The sealed compiler-embedded primitive registry has one coherent authority
decision: only declarations created by the compiler from its closed registry
carry privileged semantic identity. The decision survives the attacks below if
and only if all listed invariants remain part of the design. No implementation
exists, so parser, resolver, cache, checker, lowering, runtime, backend, and fact
channel conformance remain unproved production obligations.

## Threat boundary

The attacker controls all application and dependency source, module and package
names, build inputs, ordinary command-line options, cached dependency
artifacts, link inputs, native symbols, and ordinary library wrappers. The
attacker does not control the selected xlang compiler/runtime release itself.
Replacing or modifying the compiler is outside this static language-boundary
question; accepting a modified toolchain is a distribution-authentication
problem explicitly outside D14.

The review does not assume a standard library, signed source, trusted package
path, trusted build script, or cooperative writer.

## Adversarial cases

| Attack | Required outcome | Architectural result |
|---|---|---|
| Define a source module with the embedded module's name | It remains an ordinary module or is rejected for reserved collision | PASS: authority is compiler-created module/declaration identity, not the name |
| Define a function with the same signature and operation name | It type-checks only as an ordinary function and receives no intrinsic meaning | PASS: lowerer dispatch is identity-based |
| Re-export, alias, or wrap a real primitive | Calls remain allowed, but no new primitive identity or stronger contract appears | PASS: callability is intentionally public and definition authority does not propagate |
| Spell an internal attribute, marker, annotation, or bodyless declaration | Parser rejects it or treats it as non-authoritative ordinary syntax | PASS only if no writer-visible authority spelling exists; binding invariant recorded below |
| Enable an `official-core`, `parse-stdlib`, `builtin`, bootstrap, or unsafe command-line mode | Arbitrary source must not acquire registry origin | PASS only if no such public or dependency-controlled option exists |
| Publish a dependency at a reserved package or module path | Resolver provenance cannot turn it into the embedded module | PASS: package resolution is not authority |
| Forge or replay a serialized AST/module carrying a primitive tag | Current compiler discards serialized authority and rebinds only exact registry identities after validating all contracts and versions | PASS only with non-serializable or revalidated origin; production test required |
| Feed hand-written IR or object code to a lower compilation phase | It cannot enter the accepted-source proof/fact path as checked xlang | PASS only if raw IR/object ingestion is an external/foreign boundary and cannot claim xlang semantics |
| Export a native symbol matching a runtime helper | Binder rejects it unless selected by an authorized registry row and exact ABI/provider identity | PASS: symbol spelling alone is non-authoritative |
| Swap a runtime library or JIT implementation after checking | Loaded implementation must remain tied to the selected toolchain/runtime and exact registry contract; otherwise the build/load fails closed | PASS architecturally; final-image validation is a production obligation, not a new source authority route |
| Register an intrinsic through a compiler, backend, runtime, linker, or JIT plugin | Plugin cannot extend the privilege registry | PASS only if extension plugins are excluded from the trusted path |
| Select an unsupported target or omit a backend implementation | Compile/link rejects; it never falls back to weaker checks, effects, ABI, or ownership semantics | PASS: fail-closed target rule is explicit |
| Give constant evaluation a different implementation | It rejects evaluation or refines the same abstract contract | PASS: no independent const-semantic route |
| Let an optimizer recognize an ordinary wrapper by name or shape | Recognition may optimize only with ordinary proofs; it cannot mint primitive-only facts or semantics | PASS only if semantic facts originate from registry identity and verified contracts |
| Put a formula, opcode, syscall number, contract, proof rule, callback, or state machine in a primitive argument | The argument remains ordinary data and cannot install semantics or authority | PASS: the fixed registry row, not an argument, defines semantics |
| Use generics or reflection to manufacture the declaration type | A matching type permits calls only; identity and lowering authority are unchanged | PASS: type equality is not semantic-identity equality |
| Build without a standard library | Embedded declarations remain available; all higher abstractions remain ordinary checked source | PASS: authority has no library artifact dependency |

## Binding invariants

The architectural PASS depends on all of these invariants. Removing any one
reopens the review.

1. **Single origin.** Privileged semantic identities are created only while the
   compiler constructs the closed embedded registry.
2. **No source admission.** No token, attribute, declaration form, package,
   path, build script, environment variable, or command-line flag lets ordinary
   input enter that construction path.
3. **Identity dispatch.** Checker, constant evaluator, optimizer interface,
   lowerer, runtime binder, and backend select privileged behavior from the
   compiler-owned identity, never spelling or structural similarity.
4. **Non-transitive authority.** Importing, aliasing, re-exporting, wrapping,
   instantiating, or calling a primitive does not grant definition authority.
5. **One abstract contract.** Every physical implementation refines the same
   complete type, ownership, effect, failure, cleanup, trap, and availability
   contract.
6. **Fail-closed targets.** Missing or mismatched constant-evaluator, backend,
   runtime, ABI, provider, or target support rejects rather than weakens the
   contract.
7. **No serialized trust.** Cached or imported artifacts cannot assert registry
   origin; the current compiler recreates or exactly revalidates it against its
   own registry.
8. **No plugin extension.** Application-controlled plugins cannot add registry
   rows, implementations, proof rules, effect kinds, fact kinds, or lowering
   cases to the accepted-source path.
9. **No semantic descriptors.** Ordinary values, contracts, formulas, opcodes,
   proofs, and callbacks cannot parameterize new privileged semantics.
10. **Fact containment.** A row can mint only its reviewed fixed facts, and every
    fact-producing change receives separate hostile review before shipping.
11. **Exact runtime binding.** Native bodies are subordinate implementations of
    an authorized identity; symbols, ordinals, or ABI resemblance do not grant
    authority.
12. **Toolchain-change extension.** Adding or changing a row requires a reviewed
    compiler release change and the full cross-layer obligation set in the
    decision document.

## Attempts to reduce the mechanism further

### Delete compiler-owned identity and recognize names

Rejected. A source module, function, dependency, or native symbol can reproduce
a name. Package-manager policy would move authority out of the compiler and
would not cover no-standard-library or raw compilation consistently.

### Keep only a source attribute or bodyless declaration form

Rejected. If ordinary parsing accepts the form, the writer can request
privilege. If a hidden parser mode accepts it, that mode and its artifact-origin
decision become the real gate. Embedding the declarations removes that extra
route.

### Keep only hard-coded syntax for every operation

Not unsound, but not a reduction of semantic authority. The compiler still
needs a closed operation identity and contract table, while public grammar and
checker cases grow with the basis. The embedded registry expresses the same
authority with one ordinary call model.

### Keep only native runtime symbol imports

Rejected. Direct machine operations need no runtime body, and symbol identity
does not state checked ownership/effect semantics. A native import can implement
an already-authorized row but cannot replace the row.

### Let an official core library be trusted by path or build flag

Rejected. A path is reproducible and a flag is writer-selectable. An internal
release pipeline that marks an artifact would add a second provenance and
bootstrap TCB even though D14 excludes independently distributed privileged
extensions. The embedded registry has no need for that path.

## Residual risks and production blockers

The review does not discharge implementation details. Before production
selection or implementation, a concrete design must provide:

- exact registry schema and compiler-context identity lifecycle;
- parser/resolver/module-cache negative tests for every admission attack;
- a phase-boundary proof or validator showing identity cannot be fabricated or
  lost between checking and lowering;
- exact runtime/provider/final-image binding for runtime-backed rows;
- per-target completeness and ABI checks;
- constant-evaluation and optimization refinement tests;
- an exact inventory of fact-producing rows and separate hostile review;
- deterministic diagnostics for collisions and unsupported operations; and
- root and compiler verification gates plus adversarial mutation tests.

These are fail-closed implementation obligations, not evidence for adding a
second privilege route.

## Scope conclusion

The recommendation is sufficiently defined to let the next D14 step derive a
candidate minimal safe public capability basis under a single assumed gate:
each candidate primitive would be one fixed embedded registry row, callable by
ordinary checked code and definable only by a toolchain change. This review does
not prejudge which rows are necessary, whether the basis is complete, or
whether the owner will select the gate.
