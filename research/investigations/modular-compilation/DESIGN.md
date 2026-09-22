# Modular compilation with incremental verification and optimization

This is a proposed architecture, not implemented language behavior. Its
requirements are independent module compilation, incremental work from source
checking through executable construction, and runtime optimization that is not
artificially limited by source module boundaries. The active specification and
live design tree remain authoritative until amended. The inspected baseline is
`f3cf41d42cf6324a83c1de0b28e6c0d4e6ff9da8` (kernel v0.62).

This investigation owns the architecture, its alternatives, external evidence,
and selection criteria. Keep it as the grounds for the eventual compiler and
language decisions; revise it in place when evidence changes the proposal.

## Required outcome

1. A module's unchanged bodies do not need to be parsed, resolved, type-checked,
   ownership-checked, or proved again merely because another module changed.
2. Reuse is conditional on every semantic dependency remaining valid. Cached
   and clean checking have the same acceptance and rejection judgments. No
   timeout, cache miss, scheduling choice, or optimization result is evidence.
3. Public contracts are verified against implementations. An interface file,
   object file, fingerprint, or writer assertion is not proof of that relation.
4. Generics retain concrete specialization, source checking, and direct calls.
   Module boundaries introduce no dictionaries, boxing, or dynamic dispatch.
5. Inlining, specialization, constant propagation, layout optimization, and
   proof-derived optimizer facts remain available across module boundaries.
6. The architecture tracks semantic dependencies separately from dependencies
   introduced by optimization. An optimized caller can need new machine code
   without needing a new source proof.
7. Linking is included in the end-to-end cost and invalidation model. Reusing
   object files does not by itself establish incremental executable linking.

The performance objective is the best runtime implementation supported by
measured evidence for the target workloads, with practical incremental builds.
It is not a claim that a compiler can decide the globally fastest equivalent
program. Reusing stale inlined code, permanently disabling cross-module
optimization, and making every edited build a whole-program proof run all fail
the objective rather than providing alternative completion levels.

## Existing boundaries

- [PROG-1/2](../../../spec/kernel-spec.md#11-programs-closed-world) currently
  form one ordered source bundle, without modules or separate compilation.
- [FN-1/2/6/9](../../../spec/kernel-spec.md#8-functions-generics-contracts)
  already separate callable contracts from implementations, require symbolic
  template and concrete-instance checking, constrain instantiation cycles, and
  publish postconditions only after a recursive component has been checked.
- [DIAG-2](../../../spec/kernel-spec.md#12-diagnostics-and-checked-compilation-toolchain-floor)
  binds lowering authority and proof identities to one private checked program.
- [TYPE-6](../../../spec/kernel-spec.md#4-types) uses whole-unit declaration
  identities and visibility. Stable module-qualified identities will need to
  replace source-bundle ordinals as persistent semantic identity.
- The [driver](../../../compiler/src/driver.rs),
  [semantic checker](../../../compiler/src/semantic/check.rs), and
  [LLVM emitter](../../../compiler/src/backend/emitter.rs) currently build one
  checked inventory and one LLVM module. This design changes those boundaries;
  splitting LLVM output alone does not meet the required outcome.
- [GrowVector](../../../lib/containers/grow-vector.wf) exposes representation
  paths in both contracts and effects. Source privacy, proof visibility, and
  physical layout visibility need separate definitions.

## Architectural direction

Use explicit source modules and one persistent dependency graph whose queries
own parsing, exported declarations, checking, proof publication, instantiated
bodies, target layout, optimization planning, code generation, and linking.
The cold build evaluates the same queries as an incremental build. There is
one checker and one lowering path.

Each checked callable publishes its semantic boundary separately from its
implementation evidence. Consumers depend on the semantic value they read;
final composition also requires the currently selected implementation's valid
evidence. Changes to evidence alone must not cause transitive source reproof
when the consumer's propositions and their availability are unchanged.

Persist small module and function summaries. Load a body only for a dirty
checking query, a concrete generic instantiation that needs it, or an optimizer
that imports it. Source module identity, verification recursion groups, and
LLVM code-generation partitions serve different purposes and are not forced
to coincide.

## Discriminating validation criteria

These criteria precede any experiment for selecting this design. No performance
measurements have been made for this proposal.

### Correctness and dependency precision

For a fixed specification, compiler, target, source snapshot, dependency
selection, and optimization policy, compare a fresh build with builds reached
through edits, reversions, process restarts, different worker counts, and cache
eviction. Require identical acceptance, rule attribution and current-source
diagnostic locations; accepted executables must have the same independently
checked results. Report diagnostic ordering separately where DIAG-1 permits
implementation choices.

Include private-body changes, contract changes, nominal-layout changes,
unused and newly visible declarations, generic body and argument changes,
function-kind bindings, new and deleted dependency edges, recursive-component
merges and splits, no-heap propagation, optimizer imports, and linker inputs.
Cold and incremental runs must agree even when a newly formed recursive
component removes a previously usable postcondition.

Count query execution and invalidation by phase. A repeated no-change build
executes no body checking, proof derivation, LLVM optimization, or object
generation. An isolated nongeneric body edit with unchanged semantic boundary
and recursion membership rechecks that body's verification unit; unchanged
callers reuse source proofs. Optimizer consumers that imported the old body
must be rebuilt. Invalidating every importer solely because a whole module's
source hash changed fails the precision criterion.

### Runtime quality and construction cost

Hold WF source, algorithms, target CPU, LLVM version, runtime, profiles and
observable results fixed. Compare the current monolithic backend, separate
objects without cross-module optimization, and the proposed optimized
incremental backend. A full-LTO build is a quality comparator, not an assumed
performance optimum. Attribute every repeatable loss to a concrete missed
optimization or layout decision; do not average away a regressing workload.

Measure cold construction, unchanged rebuild, implementation-only edit,
interface edit, generic edit, and broad dependency change. Separate WF parsing,
resolution, checking/proof, specialization, optimization planning, LLVM
backend, native linking, and executable publication. Record peak memory, CPU,
wall time, loaded bodies, rebuilt partitions, and cache size. Run same-image
controls and independent output oracles under the maintained performance
method. Use real multi-module consumers plus controlled scaling cases; neither
a tiny corpus nor synthetic scale establishes large-project runtime quality.

An optimization policy is selected only after these observations distinguish
it from the alternatives. Broad rebuilds caused by actual dependency changes
are reported as such; a small source edit is not a promise of constant work.
